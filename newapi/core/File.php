<?php

declare(strict_types=1);

namespace Core;

class File
{
    private array $file;
    private string $error = '';
    private string $saveName = '';
    private array $rules = [];

    public function __construct(array $file)
    {
        $this->file = $file;
    }

    public static function create(array $file): self
    {
        return new self($file);
    }

    public function validate(array $rule): self
    {
        $this->rules = $rule;
        return $this;
    }

    public function move(string $path, string|bool $savename = true): self|false
    {
        if (($this->file['error'] ?? UPLOAD_ERR_NO_FILE) !== UPLOAD_ERR_OK) {
            $this->error = '上传失败';
            return false;
        }

        $tmp = $this->file['tmp_name'] ?? '';
        $original = $this->file['name'] ?? 'file';
        $size = (int) ($this->file['size'] ?? 0);
        $ext = strtolower(pathinfo($original, PATHINFO_EXTENSION));

        if (isset($this->rules['size']) && $size > (int) $this->rules['size']) {
            $this->error = '文件大小超出限制';
            return false;
        }
        if (isset($this->rules['ext'])) {
            $allow = array_map('strtolower', array_map('trim', explode(',', (string) $this->rules['ext'])));
            if (!in_array($ext, $allow, true)) {
                $this->error = '文件类型不允许';
                return false;
            }
        }

        $path = rtrim($path, '/\\') . DIRECTORY_SEPARATOR;
        if (!is_dir($path)) {
            mkdir($path, 0777, true);
        }

        if ($savename === true) {
            $subdir = date('Ymd') . DIRECTORY_SEPARATOR;
            if (!is_dir($path . $subdir)) {
                mkdir($path . $subdir, 0777, true);
            }
            $name = md5(uniqid((string) mt_rand(), true)) . ($ext ? '.' . $ext : '');
            $this->saveName = str_replace('\\', '/', $subdir . $name);
        } elseif (is_string($savename) && $savename !== '') {
            $this->saveName = str_replace('\\', '/', $savename);
        } else {
            $this->saveName = $original;
        }

        $target = $path . str_replace('/', DIRECTORY_SEPARATOR, $this->saveName);
        $dir = dirname($target);
        if (!is_dir($dir)) {
            mkdir($dir, 0777, true);
        }

        if (!move_uploaded_file($tmp, $target) && !rename($tmp, $target)) {
            $this->error = '保存文件失败';
            return false;
        }

        return $this;
    }

    public function getSaveName(): string
    {
        return $this->saveName;
    }

    public function getError(): string
    {
        return $this->error;
    }
}
