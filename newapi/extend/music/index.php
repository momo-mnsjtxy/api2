<?php

/**
 * Minimal Musics stub — returns empty result structure when full music pack is absent.
 */
class Musics
{
    public function __call($name, $arguments)
    {
        return [
            'error' => 'music provider unavailable',
            'method' => $name,
            'data' => [],
        ];
    }

    public function data($input, $filter = '', $type = '', $page = 1)
    {
        return [
            'error' => 'music provider unavailable',
            'data' => [],
        ];
    }

    public function search($site, $input, $class = '', $page = 1)
    {
        return $this->data($input, $class, $site, $page);
    }
}
