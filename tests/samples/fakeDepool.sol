pragma solidity >=0.6.0;
pragma AbiHeader expire;
pragma AbiHeader time;

contract FakeDePool {

    uint64 m_stake;
    address m_sender;
    address m_receiver;
    uint32 m_withdrawal;
    uint32 m_total;
    bool m_reinvest;
    uint128 m_value;

    event StakeSigningRequested(uint32 electionId, address proxy);

    function addOrdinaryStake(uint64 stake) public {
        m_stake = stake;
        m_sender = msg.sender;
        m_value = msg.value;
    }

    function addVestingStake(uint64 stake, address beneficiary, uint32 withdrawalPeriod, uint32 totalPeriod) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = stake;
        m_receiver = beneficiary;
        m_withdrawal = withdrawalPeriod;
        m_total = totalPeriod;
    }

    function addLockStake(uint64 stake, address beneficiary, uint32 withdrawalPeriod, uint32 totalPeriod) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = stake;
        m_receiver = beneficiary;
        m_withdrawal = withdrawalPeriod;
        m_total = totalPeriod;
    }

    function withdrawFromPoolingRound(uint64 withdrawValue) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = withdrawValue;
    }

    function withdrawPart(uint64 withdrawValue) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = withdrawValue;
    }

    function withdrawAll() public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_reinvest = false;
    }

    function cancelWithdrawal() public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_reinvest = true;
    }

    function transferStake(address dest, uint64 amount) public {
        m_sender = msg.sender;
        m_value = msg.value;
        m_stake = amount;
        m_receiver = dest;
    }

    function ticktock() public {
        m_sender = msg.sender;
        m_value = msg.value;

        emit StakeSigningRequested(1, address(2));
    }

    function receiveFunds() public {
        m_sender = msg.sender;
        m_value = msg.value;
    }

    function getData() public view returns (uint64 stake, address sender, address receiver,
        uint32 withdrawal, uint32 total, bool reinvest, uint128 value) {
        return (m_stake, m_sender, m_receiver, m_withdrawal, m_total, m_reinvest, m_value);
    }

    function setVestingDonor(address donor) public {
        m_receiver = donor;
    }

    function setLockDonor(address donor) public {
        m_receiver = donor;
    }

}