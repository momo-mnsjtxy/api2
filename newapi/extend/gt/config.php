<?php

// GeeTest captcha config — override via environment in production.
define('CAPTCHA_ID', getenv('GEETEST_ID') ?: 'geetest_id_placeholder');
define('PRIVATE_KEY', getenv('GEETEST_KEY') ?: 'geetest_key_placeholder');
