<?php

declare(strict_types=1);

namespace think;

use Core\Cache as CoreCache;

class Cache
{
    public static function store(string $name = 'file'): CoreCache
    {
        return CoreCache::store($name);
    }

    public static function get(string $name, mixed $default = null): mixed
    {
        return CoreCache::store('file')->get($name, $default);
    }

    public static function set(string $name, mixed $value, ?int $expire = null): bool
    {
        return CoreCache::store('file')->set($name, $value, $expire);
    }
}
