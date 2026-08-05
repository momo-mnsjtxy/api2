<?php

declare(strict_types=1);

namespace Core;

class Session
{
    private static bool $started = false;
    private static string $prefix = 'think';

    public static function start(): void
    {
        if (self::$started) {
            return;
        }
        if (session_status() !== PHP_SESSION_ACTIVE) {
            session_start();
        }
        self::$started = true;
    }

    private static function key(string $name): string
    {
        return self::$prefix . $name;
    }

    public static function set(string $name, mixed $value): void
    {
        self::start();
        $_SESSION[self::key($name)] = $value;
    }

    public static function get(string $name, mixed $default = null): mixed
    {
        self::start();
        return $_SESSION[self::key($name)] ?? $default;
    }

    public static function has(string $name): bool
    {
        self::start();
        return array_key_exists(self::key($name), $_SESSION);
    }

    public static function delete(string $name): void
    {
        self::start();
        unset($_SESSION[self::key($name)]);
    }

    public static function clear(): void
    {
        self::start();
        $_SESSION = [];
    }

    public static function all(): array
    {
        self::start();
        return $_SESSION;
    }
}
