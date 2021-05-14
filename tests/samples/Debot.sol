pragma ton-solidity >=0.35.0;
pragma AbiHeader expire;
pragma AbiHeader time;
pragma AbiHeader pubkey;

abstract contract Debot {

    uint8 constant DEBOT_ABI = 1;

    uint8 m_options;
    optional(string) m_debotAbi;
    /// @notice Deprecated. For compatibility with old DEngine.
    optional(string) m_targetAbi;
    /// @notice Deprecated. For compatibility with old DEngine.
    optional(address) m_target;

    /*
     * Public debot interface
     */

    /// @notice DeBot entry point.
    function start() public virtual;

    /// @notice Returns DeBot metadata.
    /// @return name String with name of debot, e.g. "DePool".
    /// @return version Semver version of debot, that will be converted to string like "x.y.z".
    /// @return publisher String with info about who has deployed debot to blokchain, e.g. "TON Labs".
    /// @return caption (10-20 ch.) String with short description, e.g. "Work with Smthg".
    /// @return author String with name of author of DeBot, e.g. "Ivan Ivanov".
    /// @return support Free TON address of author for questions and donations.
    /// @return hello String with first messsage with DeBot description.
    /// @return language (ISO-639) String with debot interface language, e.g. "en".
    /// @return dabi String with debot ABI.
    function getDebotInfo() public functionID(0xDEB) view virtual returns(
        string name, string version, string publisher, string caption, string author,
        address support, string hello, string language, string dabi, bytes icon);

    /// @notice Returns list of interfaces used by DeBot.
    function getRequiredInterfaces() public view virtual returns (uint256[] interfaces);

    /// @notice Returns DeBot ABI.
    /// Deprecated.
    function getDebotOptions() public view returns (uint8 options, string debotAbi, string targetAbi, address targetAddr) {
        debotAbi = m_debotAbi.hasValue() ? m_debotAbi.get() : "";
        targetAbi = m_targetAbi.hasValue() ? m_targetAbi.get() : "";
        targetAddr = m_target.hasValue() ? m_target.get() : address(0);
        options = m_options;
    }

    /// @notice Allow to set debot ABI. Do it before using debot.
    function setABI(string dabi) public {
        require(tvm.pubkey() == msg.pubkey(), 100);
        tvm.accept();
        m_options |= DEBOT_ABI;
        m_debotAbi = dabi;
    }
}