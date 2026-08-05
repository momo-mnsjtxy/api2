<?php

declare(strict_types=1);

namespace Core;

class Cookie
{
    public static function set(string $name, mixed $value, int $expire = 0, string $path = '/', string $domain = '', bool $secure = false, bool $httponly = false): void
    {
        $expireAt = $expire > 0 ? time() + $expire : 0;
        setcookie($name, (string) $value, [
            'expires' => $expireAt,
            'path' => $path,
            'domain' => $domain,
            'secure' => $secure,
            'httponly' => $httponly,
            'samesite' => 'Lax',
        ]);
        $_COOKIE[$name] = (string) $value;
    }

    public static function get(string $name, mixed $default = null): mixed
    {
        return $_COOKIE[$name] ?? $default;
    }

    public static function has(string $name): bool
    {
        return array_key_exists($name, $_COOKIE);
    }

    public static function delete(string $name, string $path = '/'): void
    {
        setcookie($name, '', [
            'expires' => time() - 3600,
            'path' => $path,
        ]);
        unset($_COOKIE[$name]);
    }
}
