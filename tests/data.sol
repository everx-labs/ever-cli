pragma solidity >=0.6.0;
pragma AbiHeader v1;

contract TestData {

    uint256 public m_id;

    /*
     * Publics
     */

    function sendTransaction(address dest, uint128 value, bool bounce) public view {
        require(msg.pubkey() == tvm.pubkey(), 101);
        tvm.accept();
        tvm.transfer(dest, value, bounce, 3);
    }

    function getId() public view returns (uint256 id) {
        id = m_id;
    }

    function getKey() public view returns (uint256 key) {
        key = tvm.pubkey();
    }
}