<?php

declare(strict_types=1);

namespace think;

use Core\Cookie as CoreCookie;

class Cookie
{
    public static function set(string $name, mixed $value, int $expire = 0): void
    {
        CoreCookie::set($name, $value, $expire);
    }

    public static function get(string $name, mixed $default = null): mixed
    {
        return CoreCookie::get($name, $default);
    }

    public static function has(string $name): bool
    {
        return CoreCookie::has($name);
    }

    public static function delete(string $name): void
    {
        CoreCookie::delete($name);
    }
}
