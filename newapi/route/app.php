<?php
use think\facade\Route;

Route::pattern([
    'name' => '\w+',
]);

Route::get('hello/:id', 'index/hello')->pattern(['id' => '\d+']);
Route::post('hello/:name', 'index/hello');
