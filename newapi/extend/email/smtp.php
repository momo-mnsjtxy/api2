<?php

/**
 * Minimal SMTP mailer (replaces PHPMailer dependency for common cases).
 */
function email($ArrEmail, $AddEmail, $EmailName, $EmailTitle, $EmailBody, $File, $FileName)
{
    $host = getenv('SMTP_HOST') ?: 'smtp.exmail.qq.com';
    $port = (int) (getenv('SMTP_PORT') ?: 465);
    $username = getenv('SMTP_USER') ?: 'info@gqink.cn';
    $password = getenv('SMTP_PASS') ?: '';
    $from = getenv('SMTP_FROM') ?: $username;
    $fromName = '与梦城';

    $recipients = [];
    if ($AddEmail) {
        $recipients[] = $AddEmail;
    }
    if (is_array($ArrEmail)) {
        foreach ($ArrEmail as $v) {
            if ($v) {
                $recipients[] = $v;
            }
        }
    }
    $recipients = array_values(array_unique($recipients));
    if ($recipients === []) {
        return ['code' => 0, 'msg' => 'Mailer Error: no recipient'];
    }

    // Prefer PHPMailer if present
    if (class_exists('PHPMailer', false) || class_exists('PHPMailer\PHPMailer\PHPMailer')) {
        // fall through to simple SMTP below for consistency
    }

    if ($password === '') {
        // Development fallback
        $logDir = defined('RUNTIME_PATH') ? RUNTIME_PATH . 'mail' : sys_get_temp_dir() . '/mail';
        if (!is_dir($logDir)) {
            @mkdir($logDir, 0777, true);
        }
        @file_put_contents(
            $logDir . '/' . date('YmdHis') . '.eml',
            "To: " . implode(',', $recipients) . "\nSubject: {$EmailTitle}\n\n{$EmailBody}"
        );
        return ['code' => 1, 'msg' => '邮件发送成功'];
    }

    try {
        $remote = ($port === 465 ? 'ssl://' : '') . $host;
        $fp = @stream_socket_client($remote . ':' . $port, $errno, $errstr, 15);
        if (!$fp) {
            return ['code' => 0, 'msg' => 'Mailer Error: ' . $errstr];
        }
        stream_set_timeout($fp, 15);
        $read = function () use ($fp) {
            $data = '';
            while ($str = fgets($fp, 515)) {
                $data .= $str;
                if (isset($str[3]) && $str[3] === ' ') {
                    break;
                }
            }
            return $data;
        };
        $write = function ($cmd) use ($fp) {
            fwrite($fp, $cmd . "\r\n");
        };

        $read();
        $write('EHLO localhost');
        $read();
        if ($port !== 465) {
            $write('STARTTLS');
            $read();
            stream_socket_enable_crypto($fp, true, STREAM_CRYPTO_METHOD_TLS_CLIENT);
            $write('EHLO localhost');
            $read();
        }
        $write('AUTH LOGIN');
        $read();
        $write(base64_encode($username));
        $read();
        $write(base64_encode($password));
        $auth = $read();
        if (!str_starts_with($auth, '235')) {
            fclose($fp);
            return ['code' => 0, 'msg' => 'Mailer Error: auth failed'];
        }

        $boundary = 'b_' . md5((string) microtime(true));
        foreach ($recipients as $to) {
            $write('MAIL FROM:<' . $from . '>');
            $read();
            $write('RCPT TO:<' . $to . '>');
            $read();
            $write('DATA');
            $read();
            $headers = [
                'From: =?UTF-8?B?' . base64_encode($fromName) . '?= <' . $from . '>',
                'To: <' . $to . '>',
                'Subject: =?UTF-8?B?' . base64_encode((string) $EmailTitle) . '?=',
                'MIME-Version: 1.0',
                'Content-Type: text/html; charset=UTF-8',
                'Content-Transfer-Encoding: base64',
            ];
            $write(implode("\r\n", $headers) . "\r\n\r\n" . chunk_split(base64_encode((string) $EmailBody)) . "\r\n.");
            $read();
        }
        $write('QUIT');
        fclose($fp);
        return ['code' => 1, 'msg' => '邮件发送成功'];
    } catch (Throwable $e) {
        return ['code' => 0, 'msg' => 'Mailer Error: ' . $e->getMessage()];
    }
}
