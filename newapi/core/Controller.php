<?php

declare(strict_types=1);

namespace Core;

class Controller
{
    protected array $viewData = [];

    public function assign(string|array $name, mixed $value = null): static
    {
        if (is_array($name)) {
            $this->viewData = array_merge($this->viewData, $name);
        } else {
            $this->viewData[$name] = $value;
        }
        View::share($this->viewData);
        return $this;
    }

    protected function fetch(?string $template = null, array $vars = []): string
    {
        return View::make($template, array_merge($this->viewData, $vars));
    }

    public function success(string $msg = '', string $url = '', int $wait = 1): never
    {
        $target = $url !== '' ? url($url) : 'javascript:history.back(-1);';
        $this->resultPage(true, $msg, $target, $wait);
    }

    public function error(string $msg = '', string $url = '', int $wait = 3): never
    {
        $target = $url !== '' ? url($url) : 'javascript:history.back(-1);';
        $this->resultPage(false, $msg, $target, $wait);
    }

    public function redirect(string $url): static
    {
        $target = preg_match('#^https?://#i', $url) ? $url : url($url);
        header('Location: ' . $target);
        exit;
    }

    private function resultPage(bool $ok, string $msg, string $url, int $wait): never
    {
        $title = $ok ? '成功' : '错误';
        $color = $ok ? '#16a34a' : '#dc2626';
        $msg = htmlspecialchars($msg, ENT_QUOTES, 'UTF-8');
        $urlAttr = htmlspecialchars($url, ENT_QUOTES, 'UTF-8');
        echo <<<HTML
<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta http-equiv="refresh" content="{$wait};url={$urlAttr}">
<title>{$title}</title>
<style>
body{font-family:system-ui,sans-serif;background:#f8fafc;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0}
.box{background:#fff;border:1px solid #e2e8f0;border-radius:12px;padding:32px 40px;text-align:center;max-width:480px}
h1{color:{$color};font-size:22px;margin:0 0 12px}
p{color:#475569;margin:0 0 18px}
a{color:#2563eb;text-decoration:none}
</style>
</head>
<body>
<div class="box">
  <h1>{$title}</h1>
  <p>{$msg}</p>
  <p><a href="{$urlAttr}">立即跳转</a>（{$wait} 秒后自动跳转）</p>
</div>
</body>
</html>
HTML;
        exit;
    }
}
