<?php
class music163 {
    private $ENCRYPT_N = '00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7';
    private $ENCRYPT_NONCE = '0CoJUm6Qyw8W8jud';
    private $ENCRYPT_E='65537';
    private $AES_VI='0102030405060708';
    protected $_USERAGENT='Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/35.0.1916.157 Safari/537.36';
    protected $_REFERER='http://music.163.com/';
    // key
    protected $secretKey='TA3YiYCfY2dDJQgg';
    protected $encSecKey='84ca47bca10bad09a6b04c5c927ef077d9b9f1e37098aa3eac6ea70eb59df0aa28b691b7e75e4f1f9831754919ea784c8f74fbfadf2898b0be17849fd656060162857830e241aba44991601f137624094c114ea8d17bce815b0cd4e5b8e2fbaba978c6d1d14dc3d1faf852bdd28818031ccdaaa13a6018e1024e2aae98844210';


    public function  lyric($song_id){
        $api = "http://music.163.com/api/song/lyric?os=pc&id=$song_id&lv=-1&kv=-1&tv=-1";
        $jsondata = $this->get_url($song_id,$api);
        $lyric = @$jsondata['lrc']['lyric'];
        return $lyric;
    }

    public function song_url($song_id){
        $api = "https://music.163.com/weapi/song/enhance/player/url?csrf_token=";
        $jsondata = $this->get_url($song_id,$api);
        $dataurl = @$jsondata['data'][0]['url'];
        return $dataurl;
    }

    public function  song_info($song_id){
        $api = "http://music.163.com/api/song/detail/?id=" . $song_id . "&ids=%5B" . $song_id . "%5D";
        $jsondata = $this->get_url($song_id,$api);
        $song= @$jsondata['songs'][0];
        $song_name = $song['name'];
        foreach ( $song['artists'] as $v){
            @$artists_name=$v['name'];   //演唱
            @$artists_img1v1Url=$v['img1v1Url']; //演唱者图片
            $song_artists[]=["artists_name"=>$artists_name,"artists_img1v1Url"=>$artists_img1v1Url];
        }
        $song_type = @$song['album']['type'];   //专辑名
        $song_album = @$song['album']['name'];   //专辑名
        $song_zj = $song_type."-".$song_album;
        $song_pic = @$song['album']['blurPicUrl'];
        $song_info = ['song_name'=>$song_name,'artists_info'=>$song_artists,'song_zj'=>$song_zj,'song_pic'=>$song_pic];
        return $song_info;
    }

    public function get_url($song_id,$api,$br = 999000){
        $postData=$this->get_prepare($song_id);
        $cookie="os=pc; osver=Microsoft-Windows-10-Professional-build-10586-64bit; appver=2.0.3.131777; channel=netease; __remember_me=true";
        $header=[
            'Origin: http://music.163.com',
            'X-Real-IP: 183.30.197.115',
            'Accept-Language: q=0.8,zh-CN;q=0.6,zh;q=0.2',
            'User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36',
            'Referer: http://music.163.com/'
        ];
        $result_json = $this->http_requests($api,$postData,$cookie,$header);
        $result = $this->object_to_array(json_decode($result_json));
        return $result;
    }


    private function prepare($raw)
    {
        $data['params'] =$this->aes_encode(json_encode($raw), $this->ENCRYPT_NONCE);
        $data['params'] = $this->aes_encode($data['params'], $this->secretKey);
        $data['encSecKey'] = $this->encSecKey;
        return $data;
    }

    private function aes_encode($secretData, $secret)
    {
        return openssl_encrypt($secretData, 'aes-128-cbc', $secret, false, $this->AES_VI);
    }

    //获取加密值：params和encSecKey
    private function  get_prepare($song_id, $br = 999000){
        if (!is_array($song_id)) $song_id = [$song_id];
        $data = [
            'ids' => $song_id,
            'br' => $br,
            'csrf_token' => '',
        ];
        $result = $this->prepare($data);
        return($result);
    }

    /* public function curl_get($url)
    {
        $refer = "http://music.163.com/";
        $header[] = "Cookie: " . "appver=1.5.0.75771;";
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_HTTPHEADER, $header);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_BINARYTRANSFER, true);
        curl_setopt($ch, CURLOPT_REFERER, $refer);
        $output = curl_exec($ch);
        curl_close($ch);
        return $output;
    }*/

    /**
     * CURL 模块
     * @param  string $uri      目的地址
     * @param  string $postData POST数组
     * @param  string $cookie   携带Cookie
     * @param  string|array $header   自定义Header
     * @return string
     */

    public function http_requests($uri, $postData = '', $cookie = '', $header = '')
    {
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $uri);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        curl_setopt($ch, CURLOPT_HEADER, 0);
        curl_setopt($ch, CURLOPT_TIMEOUT, 15);
        curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false); // https请求 不验证证书和hosts
        curl_setopt($ch, CURLOPT_SSL_VERIFYHOST, false);

        if ($postData) { // post提交
            if (is_array($postData))
                $postData = http_build_query($postData);
            curl_setopt($ch, CURLOPT_POST, 1);
            curl_setopt($ch, CURLOPT_POSTFIELDS, $postData);
        }

        if ($cookie) // 伪造cookie
            curl_setopt($ch, CURLOPT_COOKIE, $cookie);

        if ($header) // 自定义header
            curl_setopt($ch, CURLOPT_HTTPHEADER, $header);

        $data = curl_exec($ch);
        curl_close($ch);
        return $data;
    }
    //object 转 array
    public function object_to_array($obj) {
        $obj = (array)$obj;
        foreach ($obj as $k => $v) {
            if (gettype($v) == 'resource') {
                return;
            }
            if (gettype($v) == 'object' || gettype($v) == 'array') {
                $obj[$k] = (array)$this->object_to_array($v);
            }
        }
        return $obj;
    }
}

//网易云音乐解析
function m_163($id){
	$song_id = $id;
	$netease = new music163();
	$lrc = $netease->lyric($song_id);
    if(!isset($lrc)){
        $lrc = "此音乐暂无歌词";
    }
    $data = [
	'song_id'=>$song_id,
	'name'=>$netease->song_info($song_id)['song_name'],
	'author'=>$netease->song_info($song_id)['artists_info'][0]['artists_name'],
	'song_zj'=>$netease->song_info($song_id)['song_zj'],
	'url'=>$netease->song_url($song_id),
	'cover'=>$netease->song_info($song_id)['song_pic'],
	'artists_img1v1Url'=>$netease->song_info($song_id)['artists_info'][0]['artists_img1v1Url'],
	'lrc'=>$lrc,
    ];
    return $data;
}
