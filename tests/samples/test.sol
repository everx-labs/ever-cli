pragma ever-solidity >= 0.64.0;
pragma AbiHeader expire;

contract Test {

	modifier checkOwnerAndAccept {
		require(msg.pubkey() == tvm.pubkey(), 102);
		tvm.accept();
		_;
	}

	function test(uint ctype, string data) public checkOwnerAndAccept {
	}

	function get(uint ctype, string data) public returns (uint) {
		return ctype + 1;
	}
}
