<?php

declare(strict_types=1);

namespace Core;

class Router
{
    public function __construct(private App $app)
    {
    }

    public function dispatch(Request $request): mixed
    {
        $path = $this->pathInfo($request);
        $segments = array_values(array_filter(explode('/', trim($path, '/')), fn ($s) => $s !== ''));

        $module = $this->normalizeModule($segments[0] ?? 'index');
        $controller = $this->normalizeController($segments[1] ?? 'Index');
        $action = $this->normalizeAction($segments[2] ?? 'index');

        // leftover path segments as numeric params (rare)
        if (count($segments) > 3) {
            for ($i = 3; $i < count($segments); $i += 2) {
                $key = $segments[$i] ?? null;
                $val = $segments[$i + 1] ?? null;
                if ($key !== null) {
                    $_GET[$key] = $val;
                }
            }
        }

        $request->setRoute($module, $controller, $action, $path);

        $class = 'app\\' . $module . '\\controller\\' . $controller;
        if (!class_exists($class)) {
            abort(404, 'Controller not found: ' . $class);
        }

        $instance = new $class();
        if (!method_exists($instance, $action)) {
            // ThinkPHP action names are case-insensitive-ish; try common variants
            $candidates = [$action, lcfirst($action), strtolower($action)];
            $found = null;
            foreach (get_class_methods($instance) as $method) {
                if (in_array(strtolower($method), array_map('strtolower', $candidates), true)) {
                    $found = $method;
                    break;
                }
            }
            if ($found === null) {
                abort(404, 'Action not found: ' . $class . '::' . $action);
            }
            $action = $found;
        }

        $ref = new \ReflectionMethod($instance, $action);
        $args = [];
        foreach ($ref->getParameters() as $param) {
            $type = $param->getType();
            if ($type instanceof \ReflectionNamedType && !$type->isBuiltin()) {
                $typeName = $type->getName();
                if ($typeName === Request::class || $typeName === \think\Request::class || $typeName === \app\Request::class) {
                    $args[] = $request;
                    continue;
                }
            }
            $name = $param->getName();
            if ($request->has($name)) {
                $args[] = $request->param($name);
            } elseif ($param->isDefaultValueAvailable()) {
                $args[] = $param->getDefaultValue();
            } else {
                $args[] = null;
            }
        }

        return $ref->invokeArgs($instance, $args);
    }

    private function pathInfo(Request $request): string
    {
        $uri = $request->server('REQUEST_URI', '/');
        $uri = parse_url($uri, PHP_URL_PATH) ?: '/';

        // support ?s=/api/v2/xxx
        $s = $request->get('s');
        if (is_string($s) && $s !== '') {
            return '/' . ltrim($s, '/');
        }

        $scriptName = str_replace('\\', '/', $request->server('SCRIPT_NAME', ''));
        $scriptDir = rtrim(str_replace('\\', '/', dirname($scriptName)), '/');

        if ($scriptDir !== '' && $scriptDir !== '/' && str_starts_with($uri, $scriptDir)) {
            $uri = substr($uri, strlen($scriptDir)) ?: '/';
        }

        // strip front controller
        $uri = preg_replace('#^/index\.php#', '', $uri) ?: '/';
        return $uri === '' ? '/' : $uri;
    }

    private function normalizeModule(string $module): string
    {
        $module = strtolower($module);
        return preg_replace('/[^a-z0-9_]/', '', $module) ?: 'index';
    }

    private function normalizeController(string $controller): string
    {
        $controller = str_replace(['-', '_'], ' ', $controller);
        $controller = str_replace(' ', '', ucwords(strtolower($controller)));
        return preg_replace('/[^A-Za-z0-9_]/', '', $controller) ?: 'Index';
    }

    private function normalizeAction(string $action): string
    {
        // Keep original case for methods like UserInfo / Music_163
        $action = trim($action);
        if ($action === '') {
            return 'index';
        }
        // Convert dash/underscore-separated only when lowercase style
        if (str_contains($action, '-') || (str_contains($action, '_') && $action === strtolower($action))) {
            $action = lcfirst(str_replace(' ', '', ucwords(str_replace(['-', '_'], ' ', $action))));
        }
        return $action;
    }
}
