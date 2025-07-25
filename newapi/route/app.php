<?php
use think\facade\Route;

Route::pattern([
    'name' => '\w+',
]);

Route::get('/', 'index/index/index');
Route::get('login', 'login/index/index');
Route::get('register', 'register/index/index');
Route::get('hello/:id', 'index/hello')->pattern(['id' => '\d+']);
Route::post('hello/:name', 'index/hello');
