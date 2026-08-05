<?php

require_once __DIR__ . '/../extend/qrcode.php';
require_once __DIR__ . '/../extend/music163.php';
require_once __DIR__ . '/../extend/email/smtp.php';
require_once __DIR__ . '/../extend/QrReader/QrReader.php';
require_once __DIR__ . '/../extend/pinyin/pinyin.php';
require_once __DIR__ . '/../extend/music/index.php';
require_once __DIR__ . '/../extend/gt/class.geetestlib.php';
require_once __DIR__ . '/../extend/gt/config.php';
require_once __DIR__ . '/../extend/mobile/PhoneLocation.php';

//获取浏览器
function get_bro($sys = ""){
     if(!$sys){
         $sys = $_SERVER['HTTP_USER_AGENT'] ?? '';
     }
     if (stripos($sys, "Firefox/") > 0) {
         preg_match("/Firefox\/([^;)]+)+/i", $sys, $b);
         $exp[0] = "Firefox";
         $exp[1] = $b[1] ?? '';
     } elseif (stripos($sys, "Maxthon") > 0) {
         preg_match("/Maxthon\/([\d\.]+)/", $sys, $aoyou);
         $exp[0] = "傲游";
         $exp[1] = $aoyou[1] ?? '';
     } elseif (stripos($sys, "MSIE") > 0) {
         preg_match("/MSIE\s+([^;)]+)+/i", $sys, $ie);
         $exp[0] = "IE";
         $exp[1] = $ie[1] ?? '';
     } elseif (stripos($sys, "OPR") > 0) {
         preg_match("/OPR\/([\d\.]+)/", $sys, $opera);
         $exp[0] = "Opera";
         $exp[1] = $opera[1] ?? '';
     } elseif(stripos($sys, "Edge") > 0) {
         preg_match("/Edge\/([\d\.]+)/", $sys, $Edge);
         $exp[0] = "Edge";
         $exp[1] = $Edge[1] ?? '';
     } elseif (stripos($sys, "Chrome") > 0) {
         preg_match("/Chrome\/([\d\.]+)/", $sys, $google);
         $exp[0] = "Chrome";
         $exp[1] = $google[1] ?? '';
     } elseif(stripos($sys,'rv:')>0 && stripos($sys,'Gecko')>0){
         preg_match("/rv:([\d\.]+)/", $sys, $IE);
         $exp[0] = "IE";
         $exp[1] = $IE[1] ?? '';
     } elseif(stripos($sys,'Safari')>0){
         preg_match('#Safari/([a-zA-Z0-9.]+)#i', $sys, $Safari);
         $exp[0] = "Safari";
         $exp[1] = $Safari[1] ?? '';
     }else {
        $exp[0] = "未知";
        $exp[1] = "版本";
     }
     return $exp[0].'('.$exp[1].')';
}

//获取操作系统
function get_os_info($ua = "") {
    if(!$ua){
         $ua = $_SERVER['HTTP_USER_AGENT'] ?? '';
    }
	$title = '未知';
	$icon = '未知';
	if ( preg_match('/win/i', $ua) ) {
		if ( preg_match( '/Windows NT 10.0/i', $ua ) ) {
			$title = "Windows 10";
			$icon = "windows_win10";
		} elseif ( preg_match( '/Windows NT 6.1/i', $ua ) ) {
			$title = "Windows 7";
			$icon = "windows_win7";
		} elseif ( preg_match( '/Windows NT 5.1/i', $ua ) ) {
			$title = "Windows XP";
			$icon = "windows";
		} elseif ( preg_match( '/Windows NT 6.2/i', $ua ) ) {
			$title = "Windows 8";
			$icon = "windows_win8";
		} elseif ( preg_match( '/Windows NT 6.3/i', $ua ) ) {
			$title = "Windows 8.1";
			$icon = "windows_win8";
		} elseif ( preg_match( '/Windows NT 6.0/i', $ua ) ) {
			$title = "Windows Vista";
			$icon = "windows_vista";
		} elseif ( preg_match( '/Windows NT 5.2/i', $ua ) ) {
			if ( preg_match( '/Win64/i', $ua ) )
				$title = "Windows XP 64 bit";
			else
				$title = "Windows Server 2003";
			$icon = 'windows';
		} elseif ( preg_match('/Windows Phone/i', $ua ) ) {
			$matches = explode(';',$ua);
			$title = $matches[2] ?? 'Windows Phone';
			$icon = "windows_phone";
		}
	}
	elseif ( preg_match( '#iPod.*.CPU.([a-zA-Z0-9.( _)]+)#i', $ua, $matches ) ) {
		$title = "iPod ".$matches[1];
		$icon = "iphone";
	}
	elseif ( preg_match( '#iPhone OS ([a-zA-Z0-9.( _)]+)#i', $ua, $matches ) ) {
		$title = "iPhone";
		$icon = "iPhone";
	}
	elseif ( preg_match( '#iPad.*.CPU.([a-zA-Z0-9.( _)]+)#i', $ua, $matches ) ) {
		$title = "iPad ";
		$icon = "ipad";
	}
	elseif ( preg_match( '/Mac OS X.([0-9. _]+)/i', $ua, $matches ) ) {
		$title = "Mac OSX ";
		$icon = "macos";
	}
	elseif ( preg_match( '/Macintosh/i', $ua ) ) {
		$title = "Mac OS";
		$icon = "macos";
	}
	elseif ( preg_match( '/CrOS/i', $ua ) ){
		$title = "Google Chrome OS";
		$icon = "chrome";
	}
	elseif ( preg_match( '/Linux/i', $ua ) ) {
		$title = 'Linux';
		$icon = 'linux';
		if ( preg_match( '/Android.([0-9. _]+)/i', $ua, $matches ) ) {
			$title = $matches[0];
			$icon = "android";
		} elseif ( preg_match( '#Ubuntu#i', $ua ) ) {
			$title = "Ubuntu Linux";
			$icon = "ubuntu";
		} elseif ( preg_match( '#Debian#i', $ua ) ) {
			$title = "Debian GNU/Linux";
			$icon = "debian";
		} elseif ( preg_match( '#Fedora#i', $ua ) ) {
			$title = "Fedora Linux";
			$icon = "fedora";
		}
	}
	return array( $title, $icon );
}

function get_ip() {
    if (isset($_SERVER)) {
        if (isset($_SERVER['HTTP_X_FORWARDED_FOR'])) {
            $realip = $_SERVER['HTTP_X_FORWARDED_FOR'];
        } elseif (isset($_SERVER['HTTP_CLIENT_IP'])) {
            $realip = $_SERVER['HTTP_CLIENT_IP'];
        } else {
            $realip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
        }
    } else {
        if (getenv("HTTP_X_FORWARDED_FOR")) {
            $realip = getenv("HTTP_X_FORWARDED_FOR");
        } elseif (getenv("HTTP_CLIENT_IP")) {
            $realip = getenv("HTTP_CLIENT_IP");
        } else {
            $realip = getenv("REMOTE_ADDR") ?: '0.0.0.0';
        }
    }
    // XFF may contain multiple IPs
    if (str_contains((string) $realip, ',')) {
        $realip = trim(explode(',', (string) $realip)[0]);
    }
    return $realip;
}

function GET_JSON($Geturl){
	$curl = curl_init();
	curl_setopt($curl, CURLOPT_URL, $Geturl);
	curl_setopt($curl, CURLOPT_HEADER, 0);
	curl_setopt($curl, CURLOPT_RETURNTRANSFER, 1);
	curl_setopt($curl, CURLOPT_POST, 1);
	curl_setopt($curl, CURLOPT_SSL_VERIFYPEER, false);
	curl_setopt($curl, CURLOPT_SSL_VERIFYHOST, false);
	curl_setopt($curl, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_1_1);
	curl_setopt($curl, CURLOPT_POSTFIELDS, []);
	$data = curl_exec($curl);
	curl_close($curl);
	$data = json_decode(trim((string) $data, chr(239).chr(187).chr(191)), true);
	return $data;
}

function Copyright(){
	return ['name' => '与梦城' , 'url' => 'https://www.gqink.cn' , 'time' => date("Y-m-d H:i:s")];
}

function INT($type,$data){
	switch ($type) {
		case 'json':
			return json($data);
		case 'xml':
			return xml($data);
		default:
			return json($data);
	}
}
