<?php

declare(strict_types=1);

define('ROOT_PATH', dirname(__DIR__) . DIRECTORY_SEPARATOR);

require ROOT_PATH . 'vendor/autoload.php';

$app = new Core\App(ROOT_PATH);
$app->bootstrap()->run();
