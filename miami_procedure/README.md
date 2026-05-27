Procedure Name: cex_prop_amm_reprice
Type: Production-ready demo procedure for large-stage live presentation.
Business Goal:  Show a real PropAMM / CEX-informed market-making flow that top Solana traders and liquidity providers already run today. In one single JSON-RPC call the procedure does the full trading loop: receives CEX signal → reads DEX state → proprietary pricing decision → builds + submits transaction. On stage: 1 click → live logs + transaction lands in < 400 ms (visible wow effect).
PropAMM is a professional market maker that runs its own liquidity pool and continuously updates the pool price using real-time CEX feeds (like Binance). This gives traders tighter spreads and better execution prices, while the PropAMM earns higher trading fees with much lower risk.

What the Input Simulates (Business View): The input represents a real-time price alert from Binance (as if a webhook or listener received it). Presenter on stage clicks a button → e.g. “Simulate SOL +2 % pump on Binance”. Client then sends these parameters:

pair: "SOL/USDC" (fixed test pair that always succeeds)
cex_price: new Binance price (e.g. 148.50)
timestamp: current time
signal_strength: 0.0–1.0 (how strong the signal is; default 0.8)
What the Procedure Must Do:

Receive CEX Signal Procedure starts immediately after the button click (simulates external CEX listener).
Read Phase

Fetch current price from exactly two DEX pools: 1) Raydium, 2) Orca Whirlpool (SOL/USDC)
Calculate weighted on-chain price as fallback.

Compute / Decision Phase

Compare CEX price vs. on-chain weighted price.
Calculate new “fair” price using simple model: 70 % CEX price + 30 % DEX weighted price.
Decide action:
If price difference > 0.8 % → submit oracle price update to our demo PropAMM smart contract
Else → no_action

Build + Submit TX

Build real transaction
Direction and amount based on the price signal
Add priority fees
No simulation step (pure speed).

Return

tx_signature
new_fair_price
Demo-Friendly Requirements

Always use SOL/USDC as the test pair (guaranteed success).
Return rich data so the audience clearly sees “what just happened”.
Triggered from simple web button on stage (client sends JWT + params).
The input need to be the way that it leads to TX submission
The demo requires having building mini smart contract:
The demo PropAMM oracle smart contract is a minimal Solana program that stores the current fair price for the SOL/USDC pair in a single on-chain account and provides one simple update_price instruction that allows an authorized signer to update that price.

## Dev notes minimal version (memo)
### Deviations from the requirements

* Input timestamp removed
    * I get it is observability, but Viktor will not generate new timestamp on stage
    and since it is not needed for the procedure to work I decided to simplify the Input
* Input signal strength removed
    * Not sure what was the intended use
    * I could use it as DEX vs CEX weight in new_true_price calculation
* DEX price 
    * Used Orca pool and Pyth V2 Oracle price 
* Used memo tx instead of custom onchain PropAMM smart contract 


