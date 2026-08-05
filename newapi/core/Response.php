<?php

declare(strict_types=1);

namespace Core;

class Response
{
    private mixed $content;
    private int $status;
    private array $headers;

    public function __construct(mixed $content = '', int $status = 200, array $headers = [])
    {
        $this->content = $content;
        $this->status = $status;
        $this->headers = $headers;
    }

    public static function json(mixed $data, int $code = 200): self
    {
        return new self(
            json_encode($data, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES),
            $code,
            ['Content-Type' => 'application/json; charset=utf-8']
        );
    }

    public static function xml(mixed $data, int $code = 200): self
    {
        return new self(self::arrayToXml($data), $code, [
            'Content-Type' => 'text/xml; charset=utf-8',
        ]);
    }

    public static function html(string $html, int $code = 200): self
    {
        return new self($html, $code, ['Content-Type' => 'text/html; charset=utf-8']);
    }

    public function send(): void
    {
        http_response_code($this->status);
        foreach ($this->headers as $name => $value) {
            header($name . ': ' . $value);
        }
        echo (string) $this->content;
    }

    public function getContent(): mixed
    {
        return $this->content;
    }

    public function __toString(): string
    {
        return (string) $this->content;
    }

    private static function arrayToXml(mixed $data, string $root = 'root'): string
    {
        $xml = '<' . $root . '>';
        $xml .= self::buildXml($data);
        $xml .= '</' . $root . '>';
        return $xml;
    }

    private static function buildXml(mixed $data): string
    {
        if (!is_array($data)) {
            return htmlspecialchars((string) $data, ENT_XML1 | ENT_QUOTES, 'UTF-8');
        }
        $xml = '';
        foreach ($data as $key => $val) {
            if (is_numeric($key)) {
                $key = 'item' . $key;
            }
            $key = preg_replace('/[^a-zA-Z0-9_\-]/', '_', (string) $key) ?: 'item';
            $xml .= '<' . $key . '>';
            $xml .= is_array($val) ? self::buildXml($val) : htmlspecialchars((string) $val, ENT_XML1 | ENT_QUOTES, 'UTF-8');
            $xml .= '</' . $key . '>';
        }
        return $xml;
    }
}
