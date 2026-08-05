<?php

declare(strict_types=1);

use Core\App;
use Core\Cache;
use Core\Cookie;
use Core\Request;
use Core\Response;
use Core\Session;
use Core\View;

if (!defined('DS')) {
    define('DS', DIRECTORY_SEPARATOR);
}

function app(?string $abstract = null): mixed
{
    $app = App::getInstance();
    return $abstract === null ? $app : $app->make($abstract);
}

function config(?string $key = null, mixed $default = null): mixed
{
    return App::getInstance()->config($key, $default);
}

function request(): Request
{
    return App::getInstance()->request();
}

function session(string|array|null $name = null, mixed $value = null): mixed
{
    if (is_array($name)) {
        foreach ($name as $k => $v) {
            Session::set($k, $v);
        }
        return null;
    }
    if ($name === null) {
        return Session::all();
    }
    if (func_num_args() === 1) {
        return Session::get($name);
    }
    Session::set($name, $value);
    return null;
}

function cookie(string $name, mixed $value = '', int $expire = 0): void
{
    Cookie::set($name, $value, $expire);
}

function cache(string|array|null $name = null, mixed $value = '', int|null $expire = null): mixed
{
    if (is_array($name)) {
        foreach ($name as $k => $v) {
            Cache::set($k, $v, $expire);
        }
        return true;
    }
    if ($name === null) {
        return null;
    }
    if ('' === $value) {
        return Cache::get($name);
    }
    return Cache::set($name, $value, $expire);
}

function json(mixed $data, int $code = 200): Response
{
    return Response::json($data, $code);
}

function xml(mixed $data, int $code = 200): Response
{
    return Response::xml($data, $code);
}

function view(?string $template = null, array $vars = []): string
{
    return View::make($template, $vars);
}

function url(string $url = '', array $vars = []): string
{
    $url = trim($url, '/');
    $query = $vars ? ('?' . http_build_query($vars)) : '';
    $base = rtrim((string) config('app.app_url', ''), '/');
    if ($base === '') {
        $script = $_SERVER['SCRIPT_NAME'] ?? '/index.php';
        $base = str_replace('\\', '/', dirname($script));
        if (str_ends_with($base, '/public')) {
            $base = substr($base, 0, -7);
        }
        $base = rtrim($base, '/');
    }
    return ($base === '' ? '' : $base) . '/' . $url . $query;
}

function abort(int $code, string $message = ''): never
{
    http_response_code($code);
    header('Content-Type: text/plain; charset=utf-8');
    echo $message !== '' ? $message : (string) $code;
    exit;
}
