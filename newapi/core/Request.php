<?php

declare(strict_types=1);

namespace Core;

class Request
{
    private static ?self $instance = null;

    private array $get;
    private array $post;
    private array $request;
    private array $server;
    private array $files;
    private array $cookie;
    private string $pathInfo = '';
    private string $module = 'index';
    private string $controller = 'Index';
    private string $action = 'index';

    public function __construct(array $get, array $post, array $request, array $server, array $files, array $cookie)
    {
        $this->get = $get;
        $this->post = $post;
        $this->request = $request;
        $this->server = $server;
        $this->files = $files;
        $this->cookie = $cookie;
        self::$instance = $this;
    }

    public static function createFromGlobals(): static
    {
        return new static($_GET, $_POST, $_REQUEST, $_SERVER, $_FILES, $_COOKIE);
    }

    public static function instance(): self
    {
        return self::$instance ?? static::createFromGlobals();
    }

    public function setRoute(string $module, string $controller, string $action, string $pathInfo): void
    {
        $this->module = $module;
        $this->controller = $controller;
        $this->action = $action;
        $this->pathInfo = $pathInfo;
    }

    public function module(): string
    {
        return $this->module;
    }

    public function controller(): string
    {
        return $this->controller;
    }

    public function action(): string
    {
        return $this->action;
    }

    public function pathinfo(): string
    {
        return $this->pathInfo;
    }

    public function method(): string
    {
        return strtoupper($this->server['REQUEST_METHOD'] ?? 'GET');
    }

    public function isGet(): bool
    {
        return $this->method() === 'GET';
    }

    public function isPost(): bool
    {
        return $this->method() === 'POST';
    }

    public function isAjax(): bool
    {
        return strtolower($this->server['HTTP_X_REQUESTED_WITH'] ?? '') === 'xmlhttprequest';
    }

    public function param(string $name = '', mixed $default = null, string $filter = ''): mixed
    {
        if ($name === '') {
            return array_merge($this->get, $this->post);
        }
        $value = $this->post[$name] ?? $this->get[$name] ?? $this->request[$name] ?? $default;
        return $this->filterValue($value, $filter);
    }

    public function get(string $name = '', mixed $default = null): mixed
    {
        if ($name === '') {
            return $this->get;
        }
        return $this->get[$name] ?? $default;
    }

    public function post(string $name = '', mixed $default = null): mixed
    {
        if ($name === '') {
            return $this->post;
        }
        return $this->post[$name] ?? $default;
    }

    public function has(string $name, string $type = 'param'): bool
    {
        return match ($type) {
            'get' => array_key_exists($name, $this->get),
            'post' => array_key_exists($name, $this->post),
            'cookie' => array_key_exists($name, $this->cookie),
            default => array_key_exists($name, $this->post)
                || array_key_exists($name, $this->get)
                || array_key_exists($name, $this->request),
        };
    }

    public function file(string $name = ''): File|array|null
    {
        if ($name === '') {
            $all = [];
            foreach ($this->files as $key => $file) {
                $all[$key] = File::create($file);
            }
            return $all;
        }
        if (!isset($this->files[$name])) {
            return null;
        }
        return File::create($this->files[$name]);
    }

    public function ip(): string
    {
        return get_ip();
    }

    public function header(string $name = '', mixed $default = null): mixed
    {
        if ($name === '') {
            return getallheaders() ?: [];
        }
        $key = 'HTTP_' . strtoupper(str_replace('-', '_', $name));
        return $this->server[$key] ?? $default;
    }

    public function server(string $name = '', mixed $default = null): mixed
    {
        if ($name === '') {
            return $this->server;
        }
        return $this->server[$name] ?? $default;
    }

    private function filterValue(mixed $value, string $filter): mixed
    {
        if ($filter === '' || $value === null) {
            return $value;
        }
        if (is_callable($filter)) {
            return $filter($value);
        }
        return $value;
    }
}
