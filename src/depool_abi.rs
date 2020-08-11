/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 */

 pub const DEPOOL_ABI: &str = r#"
 {
	"ABI version": 2,
	"header": ["time", "expire"],
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"minRoundStake","type":"uint64"},
				{"name":"proxy0","type":"address"},
				{"name":"proxy1","type":"address"},
				{"name":"proxy2","type":"address"},
				{"name":"depoolValidator","type":"address"},
				{"name":"nodeWallet","type":"address"},
				{"name":"minStake","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "getValidatorReward",
			"inputs": [
			],
			"outputs": [
				{"name":"reward","type":"uint64"}
			]
		},
		{
			"name": "getParticipantInfo",
			"inputs": [
				{"name":"addr","type":"address"}
			],
			"outputs": [
				{"name":"total","type":"uint64"},
				{"name":"available","type":"uint64"},
				{"name":"invested","type":"uint64"},
				{"name":"reinvest","type":"bool"},
				{"name":"reward","type":"uint64"},
				{"name":"pauseStake","type":"uint64"},
				{"name":"stakes","type":"map(uint64,uint64)"},
				{"name":"vestings","type":"map(uint64,uint64)"},
				{"name":"locks","type":"map(uint64,uint64)"}
			]
		},
		{
			"name": "getDePoolInfo",
			"inputs": [
			],
			"outputs": [
				{"name":"minStake","type":"uint64"},
				{"name":"minRoundStake","type":"uint64"},
				{"name":"minNodeStake","type":"uint64"},
				{"name":"interest","type":"uint64"},
				{"name":"notifyFee","type":"uint64"},
				{"name":"addFee","type":"uint64"},
				{"name":"removeFee","type":"uint64"},
				{"name":"pauseFee","type":"uint64"},
				{"name":"setReinvestFee","type":"uint64"},
				{"name":"nodeWallet","type":"address"}
			]
		},
		{
			"name": "addOrdinaryStake",
			"inputs": [
				{"name":"unusedStake","type":"uint64"},
				{"name":"reinvest","type":"bool"}
			],
			"outputs": [
			]
		},
		{
			"name": "addVestingStake",
			"inputs": [
				{"name":"beneficiary","type":"address"},
				{"name":"withdrawalPeriod","type":"uint32"},
				{"name":"totalPeriod","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "addLockStake",
			"inputs": [
				{"name":"beneficiary","type":"address"},
				{"name":"withdrawalPeriod","type":"uint32"},
				{"name":"totalPeriod","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "removeStake",
			"inputs": [
				{"name":"doRemoveFromCurrentRound","type":"bool"},
				{"name":"targetValue","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "setReinvest",
			"inputs": [
				{"name":"flag","type":"bool"}
			],
			"outputs": [
			]
		},
		{
			"name": "pauseStake",
			"inputs": [
				{"name":"amount","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "transferStake",
			"inputs": [
				{"name":"destination","type":"address"},
				{"name":"amount","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "participateInElections",
			"id": "0x4E73744B",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"validatorKey","type":"uint256"},
				{"name":"stakeAt","type":"uint32"},
				{"name":"maxFactor","type":"uint32"},
				{"name":"adnlAddr","type":"uint256"},
				{"name":"signature","type":"bytes"}
			],
			"outputs": [
			]
		},
		{
			"name": "ticktock",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "completeRoundWithChunk",
			"inputs": [
				{"name":"roundId","type":"uint64"},
				{"name":"chunkSize","type":"uint8"}
			],
			"outputs": [
			]
		},
		{
			"name": "completeRound",
			"inputs": [
				{"name":"roundId","type":"uint64"},
				{"name":"participantQty","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "onStakeAccept",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"comment","type":"uint32"},
				{"name":"elector","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "onStakeReject",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"comment","type":"uint32"},
				{"name":"elector","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "onSuccessToRecoverStake",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"elector","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "onFailToRecoverStake",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"elector","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "terminator",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "receiveFunds",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "getRounds",
			"inputs": [
			],
			"outputs": [
				{"components":[{"name":"id","type":"uint64"},{"name":"supposedElectedAt","type":"uint32"},{"name":"supposedUnfreeze","type":"uint32"},{"name":"step","type":"uint8"},{"name":"completionReason","type":"uint8"},{"name":"participantQty","type":"uint32"},{"name":"stake","type":"uint64"},{"name":"rewards","type":"uint64"},{"name":"unused","type":"uint64"},{"name":"vsetHash","type":"uint256"},{"name":"start","type":"uint64"},{"name":"end","type":"uint64"}],"name":"rounds","type":"map(uint64,tuple)"}
			]
		},
		{
			"name": "withdrawValidatorReward",
			"inputs": [
				{"name":"amount","type":"uint64"}
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
		{
			"name": "dePoolPoolClosed",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "roundStakeIsRejected",
			"inputs": [
				{"name":"queryId","type":"uint64"},
				{"name":"comment","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "RoundCompleted",
			"inputs": [
				{"components":[{"name":"id","type":"uint64"},{"name":"supposedElectedAt","type":"uint32"},{"name":"supposedUnfreeze","type":"uint32"},{"name":"step","type":"uint8"},{"name":"completionReason","type":"uint8"},{"name":"participantQty","type":"uint32"},{"name":"stake","type":"uint64"},{"name":"rewards","type":"uint64"},{"name":"unused","type":"uint64"},{"name":"vsetHash","type":"uint256"},{"name":"start","type":"uint64"},{"name":"end","type":"uint64"}],"name":"round","type":"tuple"}
			],
			"outputs": [
			]
		},
		{
			"name": "stakeSigningRequested",
			"inputs": [
				{"name":"electionId","type":"uint32"},
				{"name":"proxy","type":"address"}
			],
			"outputs": [
			]
		}
	]
}
"#;