<?php
namespace app\api\controller;
require __DIR__ . '/../../../extend/qqskey/login.class.php';
use app\Request;

class Skey
{
    public function index(Request $request){
		$login=new \qq_login();
		$do = $request->param('do');
		if($do=='checkvc'){
			$array=$login->checkvc($request->param('uin'),$request->param('tokenid'));
		}elseif($do=='dovc'){
			$array=$login->dovc($request->param('uin'),$request->param('sig'),$request->param('ans'),$request->param('cap_cd'),$request->param('sess'),$request->param('collectname'),$request->param('websig'),$request->param('cdata'),$request->param('sid'));
		}elseif($do=='getvc'){
			$array=$login->getvc($request->param('uin'),$request->param('sig'),$request->param('sess'),$request->param('sid'),$request->param('websig'));
		}elseif($do=='qqlogin'){
			$array=$login->qqlogin($request->param('uin'),$request->param('pwd'),$request->param('p'),$request->param('vcode'),$request->param('pt_verifysession'),$request->param('cookie'));
		}elseif($do=='getqrpic'){
			$array=$login->getqrpic();
		}elseif($do == 'qrlogin'){
			if($request->param('findpwd')){
				session_start();
			};
			$array=$login->qrlogin($request->param('qrsig'));
		}elseif($do == 'list'){
			$array=$login->list($request->param('uin'),$request->param('skey'),$request->param('p_skey'));
		}elseif($do == 'danxiang'){
			$array=$login->danxiang($request->param('uin'),$request->param('qq'),$request->param('skey'));
		}elseif($do == 'del'){
			$array=$login->del($request->param('uin'),$request->param('qq'),$request->param('skey'),$request->param('p_skey'));
		}elseif($do == 'authf'){
			$array=$login->authf($request->param('uin'),$request->param('skey'),$request->param('p_skey'),$request->param('ptcz'),$request->param('RK'));
		}elseif($do=='getqrpic3rd'){
			$array=$login->getqrpic3rd($request->param('daid'),$request->param('appid'));
		}elseif($do=='qrlogin3rd'){
			$array=$login->qrlogin3rd($request->param('daid'),$request->param('appid'),$request->param('qrsig'));
		}elseif($do=='idpic'){
			$array=$login->idpic();
		}elseif($do == 'idlogin'){
			if($request->param('findpwd')){
				session_start();
			};
			$array=$login->idlogin($request->param('qrsig'));
		}elseif($do == 'InfoNull'){
			$array=$login->InfoNull($request->param('uin'),$request->param('skey'),$request->param('p_skey'),$request->param('RK'));
		}elseif($do == 'NickNull'){
			$array=$login->InfoNull($request->param('uin'),$request->param('skey'),$request->param('p_skey'),$request->param('RK'));
		}
		if(is_array(@$array)){
			return INT($request->param('type'),$array);
		}
	}
}
