pragma ton-solidity >=0.47.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;
// import required DeBot interfaces and basic DeBot contract.
import "Debot.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Terminal/Terminal.sol";
import "Shttps://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/Sdk/Sdk.sol";
import "https://raw.githubusercontent.com/everx-labs/DeBot-IS-consortium/main/EncryptionBoxInput/EncryptionBoxInput.sol";

contract EncryptionBoxInputDebot is Debot {

    uint256 constant THEIR_ENC_PUBKEY = 0x2904a07c359c317ad34301e825f6184b29f14b5c6b733155e0aeb62b8bfbee04;

    bytes m_nonce;
    uint32 m_handle;
    bytes m_data;


    function start() public override {
        m_data = "Hello world!";
        Sdk.genRandom(tvm.functionId(getRandom), 12);
    }

    function getRandom(bytes buffer) public {
        m_nonce = buffer;
        Terminal.print(tvm.functionId(runEncryptionBoxInput),"run EncryptionBoxInput");
    }

    function runEncryptionBoxInput() public {
        EncryptionBoxInput.getChaCha20Box(tvm.functionId(chachaBoxHandle),"run chacha",m_nonce);
    }

    function chachaBoxHandle(uint32 handle) public {
        m_handle = handle;
        Sdk.encrypt(tvm.functionId(decrypt), m_handle, m_data);
    }

    function decrypt(uint32 result, bytes encrypted) public {
        Sdk.decrypt(tvm.functionId(checkResult), m_handle, encrypted);
    }

    function checkResult(uint32 result, bytes decrypted) public {
        if(string(decrypted) == string(m_data)) {
            Terminal.print(tvm.functionId(regenerate),"ChaCha works");
        }
    }

    function regenerate() public {
        Sdk.genRandom(tvm.functionId(getSecretNaClBox), 24);
    }

    function getSecretNaClBox(bytes buffer) public {
        m_nonce = buffer;
        EncryptionBoxInput.getNaclSecretBox(tvm.functionId(naclSecretBoxHandle),"run secret naclbox",m_nonce);
    }

    function naclSecretBoxHandle(uint32 handle) public {
        m_handle = handle;
        Sdk.encrypt(tvm.functionId(decryptSecretNaCl), m_handle, m_data);
    }

    function decryptSecretNaCl(uint32 result, bytes encrypted) public {
        Sdk.decrypt(tvm.functionId(checkSecretNaClResult), m_handle, encrypted);
    }

    function checkSecretNaClResult(uint32 result, bytes decrypted) public {
        if(string(decrypted) == string(m_data)) {
            Terminal.print(tvm.functionId(getNaClBox),"SecretNaCl works");
        }
    }

    function getNaClBox() public {
        EncryptionBoxInput.getNaclBox(tvm.functionId(naclBoxHandle),"run naclbox",m_nonce,THEIR_ENC_PUBKEY);
    }

    function naclBoxHandle(uint32 handle) public {
        m_handle = handle;
        Sdk.encrypt(tvm.functionId(decryptNaCl), m_handle, m_data);
    }

    function decryptNaCl(uint32 result, bytes encrypted) public {
        Sdk.decrypt(tvm.functionId(checkNaClResult), m_handle, encrypted);
    }

    function checkNaClResult(uint32 result, bytes decrypted) public {
        if(string(decrypted) == string(m_data)) {
            Terminal.print(0,"NaCl works");
        }
    }

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
