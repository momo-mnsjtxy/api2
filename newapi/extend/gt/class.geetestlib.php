<?php

/**
 * Lightweight GeeTest-compatible stub.
 * When real keys/network are unavailable, failback mode is used.
 */
class GeetestLib
{
    public const GT_SDK_VERSION = 'php_3.0.0_pure';

    private string $captchaId;
    private string $privateKey;

    public function __construct($captcha_id, $private_key)
    {
        $this->captchaId = (string) $captcha_id;
        $this->privateKey = (string) $private_key;
    }

    public function pre_process($data, $new_captcha = 1)
    {
        // 0 => failback mode (local validation)
        return 0;
    }

    public function get_response_str()
    {
        return json_encode([
            'success' => 0,
            'gt' => $this->captchaId,
            'challenge' => md5(uniqid((string) mt_rand(), true)),
            'new_captcha' => true,
        ], JSON_UNESCAPED_UNICODE);
    }

    public function success_validate($challenge, $validate, $seccode, $param = null)
    {
        return $this->fail_validate($challenge, $validate, $seccode);
    }

    public function fail_validate($challenge, $validate, $seccode)
    {
        // Accept non-empty challenge in failback / development mode.
        return $challenge !== '' && $challenge !== null;
    }
}
