<?php

declare(strict_types=1);

namespace Core;

class View
{
    private static array $shared = [];

    public static function share(array $vars): void
    {
        self::$shared = array_merge(self::$shared, $vars);
    }

    public static function flush(): void
    {
        self::$shared = [];
    }

    public static function make(?string $template = null, array $vars = []): string
    {
        $request = App::getInstance()->request();
        $module = $request->module();
        $controller = strtolower($request->controller());
        $action = $request->action();

        if ($template === null || $template === '') {
            $template = $module . '/' . $controller . '/' . $action;
        }

        $template = str_replace(['.', '\\'], '/', $template);
        if (!str_contains($template, '/')) {
            $template = $module . '/' . $controller . '/' . $template;
        }

        $file = self::resolveTemplate($template);
        if (!is_file($file)) {
            throw new \RuntimeException('Template not found: ' . $template);
        }

        $compiled = self::compileFile($file, array_merge(self::$shared, $vars));
        return $compiled;
    }

    private static function resolveTemplate(string $template): string
    {
        // support module/controller/action and module/view/...
        $parts = explode('/', trim($template, '/'));
        if (count($parts) >= 3) {
            [$module, $controller, $action] = $parts;
            $candidates = [
                APP_PATH . $module . '/view/' . $controller . '/' . $action . '.html',
                APP_PATH . $module . '/view/' . $action . '.html',
            ];
        } elseif (count($parts) === 2) {
            [$module, $name] = $parts;
            $candidates = [
                APP_PATH . $module . '/view/' . $name . '.html',
            ];
        } else {
            $candidates = [APP_PATH . $template . '.html'];
        }

        foreach ($candidates as $candidate) {
            if (is_file($candidate)) {
                return $candidate;
            }
        }
        return $candidates[0];
    }

    private static function compileFile(string $file, array $vars): string
    {
        $content = (string) file_get_contents($file);
        $content = self::parseIncludes($content, $vars, dirname($file));
        $php = self::compile($content);

        $cacheDir = RUNTIME_PATH . 'cache' . DIRECTORY_SEPARATOR . 'views' . DIRECTORY_SEPARATOR;
        if (!is_dir($cacheDir)) {
            mkdir($cacheDir, 0777, true);
        }
        $cacheFile = $cacheDir . md5($file . $php) . '.php';
        if (!is_file($cacheFile)) {
            file_put_contents($cacheFile, $php);
        }

        extract($vars, EXTR_OVERWRITE);
        ob_start();
        include $cacheFile;
        return (string) ob_get_clean();
    }

    private static function parseIncludes(string $content, array &$vars, string $baseDir): string
    {
        return (string) preg_replace_callback(
            '/\{include\s+file\s*=\s*"([^"]+)"([^}]*)\/?\}/i',
            function ($m) use (&$vars, $baseDir) {
                $file = $m[1];
                $attrs = $m[2] ?? '';
                $path = self::resolveInclude($file, $baseDir);
                if (!is_file($path)) {
                    return '<!-- include missing: ' . htmlspecialchars($file) . ' -->';
                }
                $included = (string) file_get_contents($path);
                // ThinkPHP include attrs become [title]/[keywords] placeholders
                if (preg_match_all('/(\w+)\s*=\s*"([^"]*)"/', $attrs, $attrMatches, PREG_SET_ORDER)) {
                    foreach ($attrMatches as $attr) {
                        $value = $attr[2];
                        if (str_starts_with($value, '$')) {
                            $key = ltrim($value, '$');
                            $resolved = self::arrayGet($vars, $key);
                            if ($resolved === null && str_contains($key, '.')) {
                                [$root, $rest] = explode('.', $key, 2);
                                $resolved = is_array($vars[$root] ?? null) ? ($vars[$root][$rest] ?? '') : '';
                            }
                            $value = (string) ($resolved ?? '');
                        }
                        $included = str_replace('[' . $attr[1] . ']', $value, $included);
                        $vars[$attr[1]] = $value;
                    }
                }
                return self::parseIncludes($included, $vars, dirname($path));
            },
            $content
        );
    }

    private static function resolveInclude(string $file, string $baseDir): string
    {
        $file = str_replace(['.', '\\'], '/', $file);
        $candidates = [
            $baseDir . '/' . $file . '.html',
            dirname($baseDir) . '/' . $file . '.html',
            APP_PATH . 'index/view/' . $file . '.html',
        ];
        // public/header style relative to module view root
        if (str_contains($baseDir, DIRECTORY_SEPARATOR . 'view' . DIRECTORY_SEPARATOR)) {
            $viewRoot = preg_replace('#/view/.*$#', '/view', str_replace('\\', '/', $baseDir));
            $candidates[] = $viewRoot . '/' . $file . '.html';
        }
        foreach ($candidates as $candidate) {
            if (is_file($candidate)) {
                return $candidate;
            }
        }
        return $candidates[0];
    }

    private static function compile(string $content): string
    {
        // literal raw php already unlikely; convert Think tags
        $content = preg_replace('/\{volist\s+name\s*=\s*"([^"]+)"\s+id\s*=\s*"([^"]+)"\s*\}/i', '<?php foreach ((array)($\1 ?? []) as $\2): ?>', $content) ?? $content;
        $content = str_replace('{/volist}', '<?php endforeach; ?>', $content);

        $content = preg_replace_callback('/\{if\s+condition\s*=\s*"([^"]+)"\s*\}/i', function ($m) {
            return '<?php if (' . self::compileCondition($m[1]) . '): ?>';
        }, $content) ?? $content;
        $content = str_replace('{else /}', '<?php else: ?>', $content);
        $content = str_replace('{else/}', '<?php else: ?>', $content);
        $content = str_replace('{/if}', '<?php endif; ?>', $content);

        // {:expr} or {:url(...)} — also convert $a.b inside expressions
        $content = preg_replace_callback('/\{:([^}]+)\}/', function ($m) {
            $expr = preg_replace_callback('/\$[a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*)+/', function ($mm) {
                return self::compileVar($mm[0]);
            }, $m[1]) ?? $m[1];
            return '<?php echo ' . $expr . '; ?>';
        }, $content) ?? $content;

        // {$var.xxx} / {$var['x']} / {$var}
        $content = preg_replace_callback('/\{(\$[a-zA-Z_][a-zA-Z0-9_\'\"\.\[\]\-]*)\}/', function ($m) {
            return '<?php echo ' . self::compileVar($m[1]) . '; ?>';
        }, $content) ?? $content;

        return "<?php /* compiled view */ ?>\n" . $content;
    }

    private static function compileCondition(string $condition): string
    {
        $condition = trim($condition);
        $condition = preg_replace('/\beq\b/i', '==', $condition) ?? $condition;
        $condition = preg_replace('/\bneq\b/i', '!=', $condition) ?? $condition;
        $condition = preg_replace('/\bgt\b/i', '>', $condition) ?? $condition;
        $condition = preg_replace('/\blt\b/i', '<', $condition) ?? $condition;
        $condition = preg_replace('/\begt\b/i', '>=', $condition) ?? $condition;
        $condition = preg_replace('/\belt\b/i', '<=', $condition) ?? $condition;
        $condition = preg_replace_callback('/\$[a-zA-Z_][a-zA-Z0-9_\'\"\.\[\]\-]*/', function ($m) {
            return self::compileVar($m[0]);
        }, $condition) ?? $condition;
        return $condition;
    }

    private static function compileVar(string $var): string
    {
        // $user.UID -> $user['UID'] ; keep existing ['x']
        if (!str_contains($var, '.')) {
            return $var;
        }
        $parts = explode('.', $var);
        $code = array_shift($parts);
        foreach ($parts as $part) {
            if (str_contains($part, '[')) {
                $code .= '.' . $part;
            } else {
                $code .= "['" . $part . "']";
            }
        }
        // fix accidental $user['UID'].Keyword style if mixed - rare
        return $code;
    }

    private static function arrayGet(array $data, string $key): mixed
    {
        if (array_key_exists($key, $data)) {
            return $data[$key];
        }
        if (str_contains($key, '.')) {
            $value = $data;
            foreach (explode('.', $key) as $segment) {
                if (!is_array($value) || !array_key_exists($segment, $value)) {
                    return null;
                }
                $value = $value[$segment];
            }
            return $value;
        }
        return null;
    }
}
