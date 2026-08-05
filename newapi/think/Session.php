<?php

declare(strict_types=1);

namespace think;

use Core\Session as CoreSession;

class Session
{
    public static function set(string $name, mixed $value): void
    {
        CoreSession::set($name, $value);
    }

    public static function get(string $name, mixed $default = null): mixed
    {
        return CoreSession::get($name, $default);
    }

    public static function has(string $name): bool
    {
        return CoreSession::has($name);
    }

    public static function delete(string $name): void
    {
        CoreSession::delete($name);
    }

    public static function clear(): void
    {
        CoreSession::clear();
    }
}
