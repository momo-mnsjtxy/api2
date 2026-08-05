<?php

declare(strict_types=1);

namespace Core;

class App
{
    private static ?self $instance = null;

    private string $rootPath;
    private array $config = [];
    private ?Request $request = null;
    private array $bindings = [];

    public function __construct(string $rootPath)
    {
        $this->rootPath = rtrim($rootPath, DIRECTORY_SEPARATOR) . DIRECTORY_SEPARATOR;
        self::$instance = $this;
    }

    public static function getInstance(): self
    {
        if (!self::$instance) {
            throw new \RuntimeException('Application has not been bootstrapped');
        }
        return self::$instance;
    }

    public function rootPath(): string
    {
        return $this->rootPath;
    }

    public function bootstrap(): self
    {
        if (!defined('ROOT_PATH')) {
            define('ROOT_PATH', $this->rootPath);
        }
        if (!defined('APP_PATH')) {
            define('APP_PATH', $this->rootPath . 'app' . DIRECTORY_SEPARATOR);
        }
        if (!defined('RUNTIME_PATH')) {
            define('RUNTIME_PATH', $this->rootPath . 'runtime' . DIRECTORY_SEPARATOR);
        }
        if (!defined('EXTEND_PATH')) {
            define('EXTEND_PATH', $this->rootPath . 'extend' . DIRECTORY_SEPARATOR);
        }
        if (!defined('DS')) {
            define('DS', DIRECTORY_SEPARATOR);
        }

        date_default_timezone_set('Asia/Shanghai');
        $this->loadConfig();
        Session::start();
        // Prefer app\Request when available so controller typehints match.
        if (class_exists(\app\Request::class)) {
            $this->request = \app\Request::createFromGlobals();
        } else {
            $this->request = Request::createFromGlobals();
        }

        return $this;
    }

    private function loadConfig(): void
    {
        $configPath = $this->rootPath . 'config';
        foreach (glob($configPath . '/*.php') ?: [] as $file) {
            $name = basename($file, '.php');
            $this->config[$name] = require $file;
        }
    }

    public function config(?string $key = null, mixed $default = null): mixed
    {
        if ($key === null) {
            return $this->config;
        }
        $segments = explode('.', $key);
        $value = $this->config;
        foreach ($segments as $segment) {
            if (!is_array($value) || !array_key_exists($segment, $value)) {
                return $default;
            }
            $value = $value[$segment];
        }
        return $value;
    }

    public function request(): Request
    {
        if ($this->request) {
            return $this->request;
        }
        if (class_exists(\app\Request::class)) {
            return $this->request = \app\Request::createFromGlobals();
        }
        return $this->request = Request::createFromGlobals();
    }

    public function make(string $abstract): mixed
    {
        return $this->bindings[$abstract] ?? null;
    }

    public function run(): void
    {
        View::flush();
        $router = new Router($this);
        $response = $router->dispatch($this->request());
        if ($response instanceof Response) {
            $response->send();
        } elseif (is_string($response) || is_numeric($response)) {
            echo $response;
        } elseif (is_array($response)) {
            Response::json($response)->send();
        }
    }
}
