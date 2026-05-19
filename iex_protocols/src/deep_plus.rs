use super::common::{Timestamp, TradeId, Symbol, OrderId, Price, Reason, MessageParseError};
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
    pub struct ModifyFlags: u8 {
        const MaintainPriority = 1;
    }
}

macro_definition::parse_deep!(
    {Symbol, Reason}

    pub struct AddOrder 0x61 {
        pub side: enum Side{
            Buy = 0x38,
            Sell = 0x35,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub order_id: OrderId,
        pub size: u32,
        pub price: Price,
    }

    pub struct ClearBook 0x43 {
        pub reserved: u8,
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

    pub struct OrderDelete 0x52 {
        pub reserved: u8,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub order_id_reference: OrderId,
    }

    pub struct OrderExecuted 0x4C {
        pub sale_condition_flags: SaleConditionFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub order_id_reference: OrderId,
        pub size: u32,
        pub price: Price,
        pub trade_id: TradeId,
    }

    pub struct OrderModify 0x4D {
        pub modify_flags: ModifyFlags, 
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub order_id_reference: OrderId,
        pub size: u32,
        pub price: Price,
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

    pub struct SecurityEvent 0x45{
        pub security_event: enum SecurityEventIdentifier{
            OpeningProcessComplete = 0x4f,
            ClosingProcessComplete = 0x43,
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

    pub struct Trade 0x54{
        pub sale_condition_flags: SaleConditionFlags, 
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
        pub trade_id: TradeId,
    }

    pub struct TradeBreak 0x42{
        pub sale_condition_flags: SaleConditionFlags,
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub size: u32,
        pub price: Price,
        pub trade_id: TradeId,
    }

    pub struct TradingStatus 0x48 {
        pub trading_status: enum TradingStatusIdentifier{
            Halted = 0x48,
            Paused = 0x50,
            Trading = 0x54,
        },
        pub timestamp: Timestamp,
        pub symbol: Symbol,
        pub reason: Reason,
    }
);