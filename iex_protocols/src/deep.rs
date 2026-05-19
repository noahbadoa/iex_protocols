use super::common::{Timestamp, Symbol, Price, Reason, EventTime, MessageParseError};
use super::endian::Little;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SaleConditionFlags: u8 {
        const IntermarketSweep = 0x80;
        const ExtendedHours = 0x40;
        const OddLotFlag = 0x20;
        const TradeThroughExempt = 0x10;
        const SinglePriceCrossTrade = 0x08;
    }
}


bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SecurityDirectoryFlags: u8 {
        const TestSecurityFlag = 0x80;
        const WhenIssuedFlag = 0x40;
        const ETPFlag = 0x20;
    }
}


bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct QuoteFlags: u8 {
        const Halted = 0x80;
        const PreMarket = 0x40;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EventFlags: u8 {
        const Complete = 0x1;
    }
}


macro_definition::parse_deep!(
    {Symbol, Reason}

    pub struct SystemEvent 0x53{
        pub system_event: enum SystemEventIdentifier{
            StartofMessage = 0x4f,
            StartofSystemHours = 0x53,
            StartofRegularMarketHours = 0x52,
            EndofRegularMarketHours = 0x4d,
            EndofSystemHours = 0x45,
            EndofMessages = 0x43,
        },
        pub timestamp: Timestamp,
    }

    pub struct SecurityEvent 0x45{
        pub security_event: enum SecurityEventIdentifier{
            OpeningProcessComplete = 0x4f,
            ClosingProcessComplete = 0x43,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
    }

    pub struct SecurityDirectory 0x44 {
        pub flags: SecurityDirectoryFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub round_lot_size: u32,
        pub adjusted_poc_price: Price,
        pub luld_tier: enum LULDTier{
            Notapplicable = 0,
            Tier1 = 1,
            Tier2 = 2
        }
    }

    pub struct TradingStatus 0x48 {
        pub trading_status: enum TradingStatusIdentifier{
            Halted = 0x48,
            Resumed = 0x4f,
            Paused = 0x50,
            Trading = 0x54,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub reason: Reason,
    }
    
    pub struct RetailLiquidityIndicator 0x49 {
        pub retail_liquidity_indicator: enum RetailLiquidityIndicatorIdentifier{
            NotApplicable = 0x20,
            Buy = 0x41,
            Sell = 0x42,
            BuySell = 0x43,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
    }

    pub struct OperationalHaltStatus 0x4F {
        pub operational_halt_status: enum OperationalHaltStatusIdentifier{
            Halted = 0x4f,
            NotHalted = 0x4e
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
    }

    pub struct ShortSalePriceTestStatus 0x50 {
        pub short_sale_status: enum ShortSalePriceTestStatusIdentifier{
            NotinEffect = 0,
            InEffect = 1,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub detail: enum Detail{
            NoPriceTest = 0x20,
            Activated = 0x41,
            Continued = 0x43,
            Deactivated = 0x44,
            NotAvailable = 0x4e,
        },
    }

    pub struct PriceLevelUpdateBuy 0x38{
        pub event_flags: EventFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
    }

    pub struct PriceLevelUpdateSell 0x35{
        pub event_flags: EventFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
    }

    pub struct TradeReportMessage 0x54{
        pub sale_condition_flags: SaleConditionFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
        pub trade_id: i64,
    }

    pub struct OfficialPriceMessage 0x58 {
        pub price_type: enum OfficialPriceType{
            Opening = 0x51,
            Closing = 0x4D,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub official_price: Price,
    }


    pub struct TradeBreak 0x42 {
        pub sale_condition_flags: SaleConditionFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
        pub trade_id: i64,
    }

    pub struct AuctionInformation 0x41 {
        pub auction_type: enum AuctionType {
            Opening = 0x4f,
            Closing = 0x43,
            IPO = 0x49,
            Halt = 0x48,
            Volatility = 0x56,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub paired_shares: u32,
        pub reference_price: Price,
        pub indicative_clearing_price: Price,
        pub imbalance_shares: u32,
        pub imbalance_side: enum ImbalanceSide{
            Buy = 0x42,
            Sell = 0x53,
            Balanced = 0x4e
        },
        pub extension_number: u8,
        pub scheduled_auction_time: EventTime,
        pub auction_book_clearing_price: Price,
        pub collar_reference_price: Price,
        pub lower_auction_collar: Price,
        pub upper_auction_collar: Price,
    }
);