<?php

return [
    'default' => 'mysql',
    'auto_timestamp' => true,
    'datetime_format' => 'Y-m-d H:i:s',
    'prefix' => '',
    'connections' => [
        'mysql' => [
            'type' => 'mysql',
            'hostname' => getenv('DB_HOST') ?: '127.0.0.1',
            'database' => getenv('DB_DATABASE') ?: 'api',
            'username' => getenv('DB_USERNAME') ?: 'api',
            'password' => getenv('DB_PASSWORD') ?: 'api_0324',
            'hostport' => getenv('DB_PORT') ?: '3306',
            'charset' => 'utf8',
            'prefix' => '',
            'debug' => true,
        ],
    ],
];
