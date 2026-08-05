<?php

declare(strict_types=1);

namespace Core;

use PDO;

class Db
{
    private static ?PDO $pdo = null;

    public static function pdo(): PDO
    {
        if (self::$pdo instanceof PDO) {
            return self::$pdo;
        }

        $config = config('database.connections.mysql', config('database'));
        if (isset($config['connections']['mysql'])) {
            $config = $config['connections']['mysql'];
        }

        $dsn = sprintf(
            'mysql:host=%s;port=%s;dbname=%s;charset=%s',
            $config['hostname'] ?? '127.0.0.1',
            $config['hostport'] ?? '3306',
            $config['database'] ?? '',
            $config['charset'] ?? 'utf8'
        );

        self::$pdo = new PDO($dsn, $config['username'] ?? '', $config['password'] ?? '', [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
            PDO::ATTR_EMULATE_PREPARES => false,
        ]);

        return self::$pdo;
    }

    public static function table(string $table): Query
    {
        return new Query($table);
    }

    public static function name(string $table): Query
    {
        $prefix = (string) config('database.connections.mysql.prefix', config('database.prefix', ''));
        return new Query($prefix . $table);
    }

    public static function query(string $sql, array $bind = []): array
    {
        $stmt = self::pdo()->prepare($sql);
        $stmt->execute($bind);
        return $stmt->fetchAll();
    }

    public static function execute(string $sql, array $bind = []): int
    {
        $stmt = self::pdo()->prepare($sql);
        $stmt->execute($bind);
        return $stmt->rowCount();
    }
}
