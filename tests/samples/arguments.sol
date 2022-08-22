pragma ever-solidity >=0.63.0;
pragma AbiHeader expire;

contract Wallet {
    uint public val;

    function add(uint addr, uint keys, uint abi, uint method) public {
        tvm.accept();
        val += addr + keys + abi + method;
    }

    function get(uint addr, uint keys, uint abi, uint method) public view returns (uint) {
        return val + addr + keys + abi + method;
    }
}