┌─────────────────────────────────────────────────────────────┐
│ PERP TRADING ENGINE │
└─────────────────────────────────────────────────────────────┘

┌──────────────────┐
│ Market Data │
│ WebSocket │
│ (Binance) │
└────────┬─────────┘
│ Price updates, orderbook, trades
▼
┌──────────────────────────────────────────────────────────────┐
│ INGESTION LAYER │
│ ┌────────────────┐ ┌──────────────────┐ │
│ │ WS Handler │───▶│ Message Parser │ │
│ │ (tokio-tungstenite)│ (serde_json) │ │
│ └────────────────┘ └─────────┬────────┘ │
│ │ │
│ ┌────────▼────────┐ │
│ │ Market Data │ │
│ │ Normalizer │ │
│ └────────┬────────┘ │
└───────────────────────────────────┼──────────────────────────┘
│ Normalized tick
▼
┌──────────────────────────────────────────────────────────────┐
│ EVENT BUS (crossbeam-channel / tokio::mpsc) │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│ │ Tick │ │ Order │ │ Fill │ │ Position │ │
│ │ Channel │ │ Channel │ │ Channel │ │ Channel │ │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ │
└───────┼─────────────┼─────────────┼─────────────┼───────────┘
│ │ │ │
▼ ▼ ▼ ▼
┌──────────────────────────────────────────────────────────────┐
│ CORE ENGINE │
│ │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Market Data Manager │ │
│ │ - Latest price cache (DashMap<Symbol, Price>) │ │
│ │ - Orderbook manager (bid/ask levels) │ │
│ │ - VWAP, TWAP calculators │ │
│ └─────────────────┬───────────────────────────────────┘ │
│ │ │
│ ┌─────────────────▼───────────────────────────────────┐ │
│ │ Risk Engine │ │
│ │ - Position tracker (by symbol) │ │
│ │ - PnL calculator (unrealized/realized) │ │
│ │ - Margin calculator │ │
│ │ - Liquidation monitor │ │
│ │ - Max leverage enforcer │ │
│ └─────────────────┬───────────────────────────────────┘ │
│ │ │
│ ┌─────────────────▼───────────────────────────────────┐ │
│ │ Order Manager │ │
│ │ - Order validation │ │
│ │ - Order state machine (pending→filled→cancelled) │ │
│ │ - Order book (HashMap<OrderId, Order>) │ │
│ │ - Fill matcher │ │
│ └─────────────────┬───────────────────────────────────┘ │
│ │ │
│ ┌─────────────────▼───────────────────────────────────┐ │
│ │ Matching Engine (if synthetic/internal) │ │
│ │ - Limit order book (binary heap for bid/ask) │ │
│ │ - Price-time priority matching │ │
│ │ - Fill generation │ │
│ └─────────────────┬───────────────────────────────────┘ │
│ │ │
│ ┌─────────────────▼───────────────────────────────────┐ │
│ │ Funding Rate Calculator │ │
│ │ - Premium calculation (mark - index) │ │
│ │ - 8-hour funding rate │ │
│ │ - Position-based funding P&L │ │
│ └──────────────────────────────────────────────────────┘ │
│ │
└───────────────────────┬───────────────────────────────────────┘
│ Orders to execute
▼
┌──────────────────────────────────────────────────────────────┐
│ EXECUTION LAYER │
│ ┌────────────────┐ ┌──────────────────┐ │
│ │ Order Router │───▶│ Binance API │ │
│ │ │ │ Client (reqwest) │ │
│ └────────────────┘ └──────────────────┘ │
└──────────────────────────────────────────────────────────────┘
│
▼
┌──────────────────────────────────────────────────────────────┐
│ PERSISTENCE LAYER │
│ ┌────────────────┐ ┌────────────────┐ ┌──────────────┐ │
│ │ Trade Logger │ │ Position DB │ │ Metrics DB │ │
│ │ (PostgreSQL/ │ │ (Redis for │ │ (TimescaleDB/│ │
│ │ TimescaleDB) │ │ real-time) │ │ Prometheus) │ │
│ └────────────────┘ └────────────────┘ └──────────────┘ │
└──────────────────────────────────────────────────────────────┘
