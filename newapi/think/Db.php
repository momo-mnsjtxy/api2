<?php

declare(strict_types=1);

namespace think;

use Core\Db as CoreDb;
use Core\Query;

class Db
{
    public static function table(string $table): Query
    {
        return CoreDb::table($table);
    }

    public static function name(string $table): Query
    {
        return CoreDb::name($table);
    }

    public static function query(string $sql, array $bind = []): array
    {
        return CoreDb::query($sql, $bind);
    }

    public static function execute(string $sql, array $bind = []): int
    {
        return CoreDb::execute($sql, $bind);
    }
}
