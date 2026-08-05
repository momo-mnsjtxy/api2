<?php

class PhoneLocation
{
    public function find($mobile)
    {
        return [
            'province' => '',
            'city' => '',
            'sp' => '',
        ];
    }
}
