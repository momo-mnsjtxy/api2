<?php

class QrReader
{
    private string $url;
    private ?string $text = null;

    public function __construct($url)
    {
        $this->url = (string) $url;
    }

    public function text()
    {
        return $this->text;
    }
}

function QrReader($url)
{
    // Without zxing extension we cannot decode; return empty string.
    $reader = new QrReader($url);
    return $reader->text();
}
