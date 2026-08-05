<?php

declare(strict_types=1);

namespace Core;

class Cache
{
    private static string $driver = 'file';
    private static ?\Redis $redis = null;
    private static bool $redisTried = false;

    public static function store(string $name = 'file'): self
    {
        $instance = new self();
        self::$driver = $name === 'redis' ? 'redis' : 'file';
        return $instance;
    }

    public function get(string $name, mixed $default = null): mixed
    {
        return self::read($name, $default);
    }

    public function set(string $name, mixed $value, int|null $expire = null): bool
    {
        return self::write($name, $value, $expire);
    }

    public function has(string $name): bool
    {
        return self::read($name, '__MISSING__') !== '__MISSING__';
    }

    public function rm(string $name): bool
    {
        return self::delete($name);
    }

    public static function getStatic(string $name, mixed $default = null): mixed
    {
        return self::read($name, $default);
    }

    public static function setStatic(string $name, mixed $value, int|null $expire = null): bool
    {
        return self::write($name, $value, $expire);
    }

    private static function read(string $name, mixed $default = null): mixed
    {
        if (self::$driver === 'redis') {
            $redis = self::redis();
            if ($redis) {
                $value = $redis->get(self::key($name));
                if ($value === false) {
                    // fall through to file cache
                } else {
                    return self::unserialize((string) $value);
                }
            }
        }

        $file = self::filePath($name);
        if (!is_file($file)) {
            return $default;
        }
        $payload = @unserialize((string) file_get_contents($file));
        if (!is_array($payload) || !array_key_exists('expire', $payload) || !array_key_exists('data', $payload)) {
            return $default;
        }
        if ($payload['expire'] > 0 && $payload['expire'] < time()) {
            @unlink($file);
            return $default;
        }
        return $payload['data'];
    }

    private static function write(string $name, mixed $value, int|null $expire = null): bool
    {
        $expire = $expire ?? (int) config('cache.expire', 43200);
        $serialized = self::serialize($value);

        if (self::$driver === 'redis') {
            $redis = self::redis();
            if ($redis) {
                if ($expire > 0) {
                    return (bool) $redis->setex(self::key($name), $expire, $serialized);
                }
                return (bool) $redis->set(self::key($name), $serialized);
            }
        }

        $dir = RUNTIME_PATH . 'cache' . DIRECTORY_SEPARATOR;
        if (!is_dir($dir)) {
            mkdir($dir, 0777, true);
        }
        $payload = [
            'expire' => $expire > 0 ? time() + $expire : 0,
            'data' => $value,
        ];
        return (bool) file_put_contents(self::filePath($name), serialize($payload), LOCK_EX);
    }

    private static function delete(string $name): bool
    {
        if (self::$driver === 'redis') {
            $redis = self::redis();
            if ($redis) {
                $redis->del(self::key($name));
            }
        }
        $file = self::filePath($name);
        return is_file($file) ? @unlink($file) : true;
    }

    private static function redis(): ?\Redis
    {
        if (self::$redisTried) {
            return self::$redis;
        }
        self::$redisTried = true;
        if (!class_exists(\Redis::class)) {
            return null;
        }
        try {
            $redis = new \Redis();
            $host = (string) config('cache.redis.host', '127.0.0.1');
            $port = (int) config('cache.redis.port', 6379);
            $redis->connect($host, $port, 1.5);
            self::$redis = $redis;
        } catch (\Throwable) {
            self::$redis = null;
        }
        return self::$redis;
    }

    private static function key(string $name): string
    {
        return 'api:' . $name;
    }

    private static function filePath(string $name): string
    {
        return RUNTIME_PATH . 'cache' . DIRECTORY_SEPARATOR . md5($name) . '.cache';
    }

    private static function serialize(mixed $value): string
    {
        return is_string($value) ? $value : serialize($value);
    }

    private static function unserialize(string $value): mixed
    {
        if ($value !== '' && ($value[0] === 'a' || $value[0] === 'O' || $value[0] === 's' || $value[0] === 'i' || $value[0] === 'b' || $value[0] === 'N' || $value[0] === 'd')) {
            $data = @unserialize($value);
            if ($data !== false || $value === 'b:0;') {
                return $data;
            }
        }
        return $value;
    }
}
