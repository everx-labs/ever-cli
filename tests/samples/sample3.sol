pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "Debot.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "Shttps://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/Sdk/Sdk.sol";
import "https://raw.githubusercontent.com/tonlabs/DeBot-IS-consortium/main/EncryptionBoxInput/EncryptionBoxInput.sol";

contract EncryptionBoxInputDebot is Debot {

    uint256 constant THEIR_ENC_PUBKEY = 0x2904a07c359c317ad34301e825f6184b29f14b5c6b733155e0aeb62b8bfbee04;

    bytes m_nonce;
    uint32 m_handle;


    function start() public override {
        Sdk.genRandom(tvm.functionId(getRandom), 24);
    }

    function getRandom(bytes buffer) public {
        m_nonce = buffer;
        Terminal.print(tvm.functionId(runEncryptionBoxInput),"run EncryptionBoxInput");
    }

    function runEncryptionBoxInput() public {
        //EncryptionBoxInput.getNaclBox(tvm.functionId(naclBoxHandle),"run naclbox",m_nonce,THEIR_ENC_PUBKEY);
        EncryptionBoxInput.getChaCha20Box(tvm.functionId(naclBoxHandle),"run naclbox",m_nonce);
    }

    function naclBoxHandle(uint32 handle) public {}
    /*function naclBoxHandle(uint32 handle) public {
        m_handle = handle;
        Terminal.print(tvm.functionId(encrypt),format("EncryptionBox handle {}",m_handle));
    }


    function encrypt() public {
        bytes data = "Hello world!";
        Sdk.encrypt(tvm.functionId(setEncrypted), m_handle, data);
    }

    function setEncrypted(uint32 result, bytes encrypted) public {
        Terminal.print(0,format("result {}",result));
        Terminal.print(0,format("encrypted {}",encrypted));
    }*/

    /*
    *  Implementation of DeBot
    */
    function getDebotInfo() public functionID(0xDEB) override view returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon
    ) {
        name = "EncryptionBoxInput";
        version = "0.1.2";
        publisher = "";
        caption = "Encryption Box Input.";
        author = "";
        support = address.makeAddrStd(0, 0x0);
        hello = "";
        language = "en";
        dabi = m_debotAbi.get();
        icon = "";
    }

    function getRequiredInterfaces() public view override returns (uint256[] interfaces) {
        return [ Sdk.ID, Terminal.ID, EncryptionBoxInput.ID];
    }

}
