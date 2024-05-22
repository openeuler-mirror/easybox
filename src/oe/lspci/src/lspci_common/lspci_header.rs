//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

///
pub const PCI_VENDOR_ID: u8 = 0x00;
///
pub const PCI_DEVICE_ID: u8 = 0x02;
///
pub const PCI_COMMAND: u8 = 0x04;
///
pub const PCI_COMMAND_IO: u8 = 0x1;
///
pub const PCI_COMMAND_MEMORY: u8 = 0x2;
///
pub const PCI_COMMAND_MASTER: u8 = 0x4;
///
pub const PCI_COMMAND_SPECIAL: u8 = 0x8;
///
pub const PCI_COMMAND_INVALIDATE: u8 = 0x10;
///
pub const PCI_COMMAND_VGA_PALETTE: u8 = 0x20;
///
pub const PCI_COMMAND_PARITY: u8 = 0x40;
///
pub const PCI_COMMAND_WAIT: u8 = 0x80;
///
pub const PCI_COMMAND_SERR: u16 = 0x100;
///
pub const PCI_COMMAND_FAST_BACK: u16 = 0x200;
///
pub const PCI_COMMAND_DISABLE_INTX: u16 = 0x400;
///
pub const PCI_STATUS: u8 = 0x06;
///
pub const PCI_STATUS_INTX: u8 = 0x08;
///
pub const PCI_STATUS_CAP_LIST: u8 = 0x10;
///
pub const PCI_STATUS_66MHZ: u8 = 0x20;
///
pub const PCI_STATUS_UDF: u8 = 0x40;
///
pub const PCI_STATUS_FAST_BACK: u8 = 0x80;
///
pub const PCI_STATUS_PARITY: u16 = 0x100;
///
pub const PCI_STATUS_DEVSEL_MASK: u16 = 0x600;
///
pub const PCI_STATUS_DEVSEL_FAST: u16 = 0x000;
///
pub const PCI_STATUS_DEVSEL_MEDIUM: u16 = 0x200;
///
pub const PCI_STATUS_DEVSEL_SLOW: u16 = 0x400;
///
pub const PCI_STATUS_SIG_TARGET_ABORT: u16 = 0x800;
///
pub const PCI_STATUS_REC_TARGET_ABORT: u16 = 0x1000;
///
pub const PCI_STATUS_REC_MASTER_ABORT: u16 = 0x2000;
///
pub const PCI_STATUS_SIG_SYSTEM_ERROR: u16 = 0x4000;
///
pub const PCI_STATUS_DETECTED_PARITY: u16 = 0x8000;
///
pub const PCI_CLASS_REVISION: u8 = 0x08;
///
pub const PCI_REVISION_ID: u8 = 0x08;
///
pub const PCI_CLASS_PROG: u8 = 0x09;
///
pub const PCI_CLASS_DEVICE: u8 = 0x0a;
///
pub const PCI_CACHE_LINE_SIZE: u8 = 0x0c;
///
pub const PCI_LATENCY_TIMER: u8 = 0x0d;
///
pub const PCI_HEADER_TYPE: usize = 0x0e;
///
pub const PCI_HEADER_TYPE_NORMAL: u8 = 0;
///
pub const PCI_HEADER_TYPE_BRIDGE: u8 = 1;
///
pub const PCI_HEADER_TYPE_CARDBUS: u8 = 2;
///
pub const PCI_BIST: usize = 0x0f;
///
pub const PCI_BIST_CODE_MASK: u8 = 0x0f;
///
pub const PCI_BIST_START: u8 = 0x40;
///
pub const PCI_BIST_CAPABLE: u8 = 0x80;
///
pub const PCI_BASE_ADDRESS_0: u8 = 0x10;
///
pub const PCI_BASE_ADDRESS_1: u8 = 0x14;
///
pub const PCI_BASE_ADDRESS_2: u8 = 0x18;
///
pub const PCI_BASE_ADDRESS_3: u8 = 0x1c;
///
pub const PCI_BASE_ADDRESS_4: u8 = 0x20;
///
pub const PCI_BASE_ADDRESS_5: u8 = 0x24;
///
pub const PCI_BASE_ADDRESS_SPACE: u8 = 0x01;
///
pub const PCI_BASE_ADDRESS_SPACE_IO: u8 = 0x01;
///
pub const PCI_BASE_ADDRESS_SPACE_MEMORY: u8 = 0x00;
///
pub const PCI_BASE_ADDRESS_MEM_TYPE_MASK: u8 = 0x06;
///
pub const PCI_BASE_ADDRESS_MEM_TYPE_32: u8 = 0x00;
///
pub const PCI_BASE_ADDRESS_MEM_TYPE_1M: u8 = 0x02;
///
pub const PCI_BASE_ADDRESS_MEM_TYPE_64: u8 = 0x04;
///
pub const PCI_BASE_ADDRESS_MEM_PREFETCH: u8 = 0x08;
///
pub const PCI_CARDBUS_CIS: u8 = 0x28;
///
pub const PCI_SUBSYSTEM_VENDOR_ID: u8 = 0x2c;
///
pub const PCI_SUBSYSTEM_ID: u8 = 0x2e;
///
pub const PCI_ROM_ADDRESS: u8 = 0x30;
///
pub const PCI_ROM_ADDRESS_ENABLE: u8 = 0x01;
///
pub const PCI_CAPABILITY_LIST: u8 = 0x34;
///
pub const PCI_INTERRUPT_LINE: u8 = 0x3c;
///
pub const PCI_INTERRUPT_PIN: u8 = 0x3d;
///
pub const PCI_MIN_GNT: usize = 0x3e;
///
pub const PCI_MAX_LAT: usize = 0x3f;
///
pub const PCI_PRIMARY_BUS: u8 = 0x18;
///
pub const PCI_SECONDARY_BUS: u8 = 0x19;
///
pub const PCI_SUBORDINATE_BUS: u8 = 0x1a;
///
pub const PCI_SEC_LATENCY_TIMER: u8 = 0x1b;
///
pub const PCI_IO_BASE: u8 = 0x1c;
///
pub const PCI_IO_LIMIT: u8 = 0x1d;
///
pub const PCI_IO_RANGE_TYPE_MASK: u8 = 0x0f;
///
pub const PCI_IO_RANGE_TYPE_16: u8 = 0x00;
///
pub const PCI_IO_RANGE_TYPE_32: u8 = 0x01;
///
pub const PCI_SEC_STATUS: u8 = 0x1e;
///
pub const PCI_MEMORY_BASE: u8 = 0x20;
///
pub const PCI_MEMORY_LIMIT: u8 = 0x22;
///
pub const PCI_MEMORY_RANGE_TYPE_MASK: u8 = 0x0f;
///
pub const PCI_PREF_MEMORY_BASE: u8 = 0x24;
///
pub const PCI_PREF_MEMORY_LIMIT: u8 = 0x26;
///
pub const PCI_PREF_RANGE_TYPE_MASK: u8 = 0x0f;
///
pub const PCI_PREF_RANGE_TYPE_32: u8 = 0x00;
///
pub const PCI_PREF_RANGE_TYPE_64: u8 = 0x01;
///
pub const PCI_PREF_BASE_UPPER32: u8 = 0x28;
///
pub const PCI_PREF_LIMIT_UPPER32: u8 = 0x2c;
///
pub const PCI_IO_BASE_UPPER16: u8 = 0x30;
///
pub const PCI_IO_LIMIT_UPPER16: u8 = 0x32;
///
pub const PCI_ROM_ADDRESS1: u8 = 0x38;
///
pub const PCI_BRIDGE_CONTROL: u8 = 0x3e;
///
pub const PCI_BRIDGE_CTL_PARITY: u8 = 0x01;
///
pub const PCI_BRIDGE_CTL_SERR: u8 = 0x02;
///
pub const PCI_BRIDGE_CTL_NO_ISA: u8 = 0x04;
///
pub const PCI_BRIDGE_CTL_VGA: u8 = 0x08;
///
pub const PCI_BRIDGE_CTL_VGA_16BIT: u8 = 0x10;
///
pub const PCI_BRIDGE_CTL_MASTER_ABORT: u8 = 0x20;
///
pub const PCI_BRIDGE_CTL_BUS_RESET: u8 = 0x40;
///
pub const PCI_BRIDGE_CTL_FAST_BACK: u8 = 0x80;
///
pub const PCI_BRIDGE_CTL_PRI_DISCARD_TIMER: u16 = 0x100;
///
pub const PCI_BRIDGE_CTL_SEC_DISCARD_TIMER: u16 = 0x200;
///
pub const PCI_BRIDGE_CTL_DISCARD_TIMER_STATUS: u16 = 0x400;
///
pub const PCI_BRIDGE_CTL_DISCARD_TIMER_SERR_EN: u16 = 0x800;
///
pub const PCI_CB_CAPABILITY_LIST: u8 = 0x14;
///
pub const PCI_CB_SEC_STATUS: u8 = 0x16;
///
pub const PCI_CB_PRIMARY_BUS: u8 = 0x18;
///
pub const PCI_CB_CARD_BUS: u8 = 0x19;
///
pub const PCI_CB_SUBORDINATE_BUS: u8 = 0x1a;
///
pub const PCI_CB_LATENCY_TIMER: u8 = 0x1b;
///
pub const PCI_CB_MEMORY_BASE_0: u8 = 0x1c;
///
pub const PCI_CB_MEMORY_LIMIT_0: u8 = 0x20;
///
pub const PCI_CB_MEMORY_BASE_1: u8 = 0x24;
///
pub const PCI_CB_MEMORY_LIMIT_1: u8 = 0x28;
///
pub const PCI_CB_IO_BASE_0: u8 = 0x2c;
///
pub const PCI_CB_IO_BASE_0_HI: u8 = 0x2e;
///
pub const PCI_CB_IO_LIMIT_0: u8 = 0x30;
///
pub const PCI_CB_IO_LIMIT_0_HI: u8 = 0x32;
///
pub const PCI_CB_IO_BASE_1: u8 = 0x34;
///
pub const PCI_CB_IO_BASE_1_HI: u8 = 0x36;
///
pub const PCI_CB_IO_LIMIT_1: u8 = 0x38;
///
pub const PCI_CB_IO_LIMIT_1_HI: u8 = 0x3a;
///
pub const PCI_CB_BRIDGE_CONTROL: u8 = 0x3e;
///
pub const PCI_CB_BRIDGE_CTL_PARITY: u8 = 0x01;
///
pub const PCI_CB_BRIDGE_CTL_SERR: u8 = 0x02;
///
pub const PCI_CB_BRIDGE_CTL_ISA: u8 = 0x04;
///
pub const PCI_CB_BRIDGE_CTL_VGA: u8 = 0x08;
///
pub const PCI_CB_BRIDGE_CTL_MASTER_ABORT: u8 = 0x20;
///
pub const PCI_CB_BRIDGE_CTL_CB_RESET: u8 = 0x40;
///
pub const PCI_CB_BRIDGE_CTL_16BIT_INT: u8 = 0x80;
///
pub const PCI_CB_BRIDGE_CTL_PREFETCH_MEM0: u16 = 0x100;
///
pub const PCI_CB_BRIDGE_CTL_PREFETCH_MEM1: u16 = 0x200;
///
pub const PCI_CB_BRIDGE_CTL_POST_WRITES: u16 = 0x400;
///
pub const PCI_CB_SUBSYSTEM_VENDOR_ID: u8 = 0x40;
///
pub const PCI_CB_SUBSYSTEM_ID: u8 = 0x42;
///
pub const PCI_CB_LEGACY_MODE_BASE: u8 = 0x44;
///
pub const PCI_CAP_LIST_ID: u16 = 0;
///
pub const PCI_CAP_ID_NULL: u8 = 0x00;
///
pub const PCI_CAP_ID_PM: u8 = 0x01;
///
pub const PCI_CAP_ID_AGP: u8 = 0x02;
///
pub const PCI_CAP_ID_VPD: u8 = 0x03;
///
pub const PCI_CAP_ID_SLOTID: u8 = 0x04;
///
pub const PCI_CAP_ID_MSI: u8 = 0x05;
///
pub const PCI_CAP_ID_CHSWP: u8 = 0x06;
///
pub const PCI_CAP_ID_PCIX: u8 = 0x07;
///
pub const PCI_CAP_ID_HT: u8 = 0x08;
///
pub const PCI_CAP_ID_VNDR: u8 = 0x09;
///
pub const PCI_CAP_ID_DBG: u8 = 0x0A;
///
pub const PCI_CAP_ID_CCRC: u8 = 0x0B;
///
pub const PCI_CAP_ID_HOTPLUG: u8 = 0x0C;
///
pub const PCI_CAP_ID_SSVID: u8 = 0x0D;
///
pub const PCI_CAP_ID_AGP3: u8 = 0x0E;
///
pub const PCI_CAP_ID_SECURE: u8 = 0x0F;
///
pub const PCI_CAP_ID_EXP: u8 = 0x10;
///
pub const PCI_CAP_ID_MSIX: u8 = 0x11;
///
pub const PCI_CAP_ID_SATA: u8 = 0x12;
///
pub const PCI_CAP_ID_AF: u8 = 0x13;
///
pub const PCI_CAP_ID_EA: u8 = 0x14;
///
pub const PCI_CAP_LIST_NEXT: u16 = 1;
///
pub const PCI_CAP_FLAGS: u16 = 2;
///
pub const PCI_CAP_SIZEOF: u16 = 4;
///
pub const PCI_EXT_CAP_ID_NULL: u8 = 0x00;
///
pub const PCI_EXT_CAP_ID_AER: u8 = 0x01;
///
pub const PCI_EXT_CAP_ID_VC: u8 = 0x02;
///
pub const PCI_EXT_CAP_ID_DSN: u8 = 0x03;
///
pub const PCI_EXT_CAP_ID_PB: u8 = 0x04;
///
pub const PCI_EXT_CAP_ID_RCLINK: u8 = 0x05;
///
pub const PCI_EXT_CAP_ID_RCILINK: u8 = 0x06;
///
pub const PCI_EXT_CAP_ID_RCEC: u8 = 0x07;
///
pub const PCI_EXT_CAP_ID_MFVC: u8 = 0x08;
///
pub const PCI_EXT_CAP_ID_VC2: u8 = 0x09;
///
pub const PCI_EXT_CAP_ID_RCRB: u8 = 0x0a;
///
pub const PCI_EXT_CAP_ID_VNDR: u8 = 0x0b;
///
pub const PCI_EXT_CAP_ID_ACS: u8 = 0x0d;
///
pub const PCI_EXT_CAP_ID_ARI: u8 = 0x0e;
///
pub const PCI_EXT_CAP_ID_ATS: u8 = 0x0f;
///
pub const PCI_EXT_CAP_ID_SRIOV: u8 = 0x10;
///
pub const PCI_EXT_CAP_ID_MRIOV: u8 = 0x11;
///
pub const PCI_EXT_CAP_ID_MCAST: u8 = 0x12;
///
pub const PCI_EXT_CAP_ID_PRI: u8 = 0x13;
///
pub const PCI_EXT_CAP_ID_REBAR: u8 = 0x15;
///
pub const PCI_EXT_CAP_ID_DPA: u8 = 0x16;
///
pub const PCI_EXT_CAP_ID_TPH: u8 = 0x17;
///
pub const PCI_EXT_CAP_ID_LTR: u8 = 0x18;
///
pub const PCI_EXT_CAP_ID_SECPCI: u8 = 0x19;
///
pub const PCI_EXT_CAP_ID_PMUX: u8 = 0x1a;
///
pub const PCI_EXT_CAP_ID_PASID: u8 = 0x1b;
///
pub const PCI_EXT_CAP_ID_LNR: u8 = 0x1c;
///
pub const PCI_EXT_CAP_ID_DPC: u8 = 0x1d;
///
pub const PCI_EXT_CAP_ID_L1PM: u8 = 0x1e;
///
pub const PCI_EXT_CAP_ID_PTM: u8 = 0x1f;
///
pub const PCI_EXT_CAP_ID_M_PCIE: u8 = 0x20;
///
pub const PCI_EXT_CAP_ID_FRS: u8 = 0x21;
///
pub const PCI_EXT_CAP_ID_RTR: u8 = 0x22;
///
pub const PCI_EXT_CAP_ID_DVSEC: u8 = 0x23;
///
pub const PCI_EXT_CAP_ID_VF_REBAR: u8 = 0x24;
///
pub const PCI_EXT_CAP_ID_DLNK: u8 = 0x25;
///
pub const PCI_EXT_CAP_ID_16GT: u8 = 0x26;
///
pub const PCI_EXT_CAP_ID_LMR: u8 = 0x27;
///
pub const PCI_EXT_CAP_ID_HIER_ID: u8 = 0x28;
///
pub const PCI_EXT_CAP_ID_NPEM: u8 = 0x29;
///
pub const PCI_EXT_CAP_ID_32GT: u8 = 0x2a;
///
pub const PCI_EXT_CAP_ID_DOE: u8 = 0x2e;
///
pub const PCI_EXT_CAP_ID_IDE: u8 = 0x30;
///
pub const PCI_PM_CAP_VER_MASK: u16 = 0x0007;
///
pub const PCI_PM_CAP_PME_CLOCK: u16 = 0x0008;
///
pub const PCI_PM_CAP_DSI: u16 = 0x0020;
///
pub const PCI_PM_CAP_AUX_C_MASK: u16 = 0x01c0;
///
pub const PCI_PM_CAP_D1: u16 = 0x0200;
///
pub const PCI_PM_CAP_D2: u16 = 0x0400;
///
pub const PCI_PM_CAP_PME_D0: u16 = 0x0800;
///
pub const PCI_PM_CAP_PME_D1: u16 = 0x1000;
///
pub const PCI_PM_CAP_PME_D2: u16 = 0x2000;
///
pub const PCI_PM_CAP_PME_D3_HOT: u16 = 0x4000;
///
pub const PCI_PM_CAP_PME_D3_COLD: u16 = 0x8000;
///
pub const PCI_PM_CTRL: u16 = 4;
///
pub const PCI_PM_CTRL_STATE_MASK: u16 = 0x0003;
///
pub const PCI_PM_CTRL_NO_SOFT_RST: u16 = 0x0008;
///
pub const PCI_PM_CTRL_PME_ENABLE: u16 = 0x0100;
///
pub const PCI_PM_CTRL_DATA_SEL_MASK: u16 = 0x1e00;
///
pub const PCI_PM_CTRL_DATA_SCALE_MASK: u16 = 0x6000;
///
pub const PCI_PM_CTRL_PME_STATUS: u16 = 0x8000;
///
pub const PCI_PM_PPB_EXTENSIONS: u16 = 6;
///
pub const PCI_PM_PPB_B2_B3: u8 = 0x40;
///
pub const PCI_PM_BPCC_ENABLE: u8 = 0x80;
///
pub const PCI_PM_DATA_REGISTER: u16 = 7;
///
pub const PCI_PM_SIZEOF: u16 = 8;
///
pub const PCI_AGP_VERSION: u16 = 2;
///
pub const PCI_AGP_RFU: u16 = 3;
///
pub const PCI_AGP_STATUS: u16 = 4;
///
pub const PCI_AGP_STATUS_ARQSZ_MASK: u16 = 0xe000;
///
pub const PCI_AGP_STATUS_CAL_MASK: u16 = 0x1c00;
///
pub const PCI_AGP_STATUS_SBA: u16 = 0x0200;
///
pub const PCI_AGP_STATUS_ITA_COH: u16 = 0x0100;
///
pub const PCI_AGP_STATUS_GART64: u16 = 0x0080;
///
pub const PCI_AGP_STATUS_HTRANS: u16 = 0x0040;
///
pub const PCI_AGP_STATUS_64BIT: u16 = 0x0020;
///
pub const PCI_AGP_STATUS_FW: u16 = 0x0010;
///
pub const PCI_AGP_STATUS_AGP3: u16 = 0x0008;
///
pub const PCI_AGP_STATUS_RATE4: u16 = 0x0004;
///
pub const PCI_AGP_STATUS_RATE2: u16 = 0x0002;
///
pub const PCI_AGP_STATUS_RATE1: u16 = 0x0001;
///
pub const PCI_AGP_COMMAND: u16 = 8;
///
pub const PCI_AGP_COMMAND_ARQSZ_MASK: u16 = 0xe000;
///
pub const PCI_AGP_COMMAND_CAL_MASK: u16 = 0x1c00;
///
pub const PCI_AGP_COMMAND_SBA: u16 = 0x0200;
///
pub const PCI_AGP_COMMAND_AGP: u16 = 0x0100;
///
pub const PCI_AGP_COMMAND_GART64: u16 = 0x0080;
///
pub const PCI_AGP_COMMAND_64BIT: u16 = 0x0020;
///
pub const PCI_AGP_COMMAND_FW: u16 = 0x0010;
///
pub const PCI_AGP_COMMAND_RATE4: u16 = 0x0004;
///
pub const PCI_AGP_COMMAND_RATE2: u16 = 0x0002;
///
pub const PCI_AGP_COMMAND_RATE1: u16 = 0x0001;
///
pub const PCI_AGP_SIZEOF: u16 = 12;
///
pub const PCI_VPD_ADDR: u16 = 2;
///
pub const PCI_VPD_ADDR_MASK: u16 = 0x7fff;
///
pub const PCI_VPD_ADDR_F: u16 = 0x8000;
///
pub const PCI_VPD_DATA: u16 = 4;
///
pub const PCI_SID_ESR: u16 = 2;
///
pub const PCI_SID_ESR_NSLOTS: u8 = 0x1f;
///
pub const PCI_SID_ESR_FIC: u8 = 0x20;
///
pub const PCI_SID_CHASSIS_NR: u16 = 3;
///
pub const PCI_MSI_FLAGS: u16 = 2;
///
pub const PCI_MSI_FLAGS_MASK_BIT: u16 = 0x100;
///
pub const PCI_MSI_FLAGS_64BIT: u16 = 0x080;
///
pub const PCI_MSI_FLAGS_QSIZE: u16 = 0x070;
///
pub const PCI_MSI_FLAGS_QMASK: u16 = 0x00e;
///
pub const PCI_MSI_FLAGS_ENABLE: u16 = 0x001;
///
pub const PCI_MSI_RFU: u16 = 3;
///
pub const PCI_MSI_ADDRESS_LO: u16 = 4;
///
pub const PCI_MSI_ADDRESS_HI: u16 = 8;
///
pub const PCI_MSI_DATA_32: u16 = 8;
///
pub const PCI_MSI_DATA_64: u16 = 12;
///
pub const PCI_MSI_MASK_BIT_32: u16 = 12;
///
pub const PCI_MSI_MASK_BIT_64: u16 = 16;
///
pub const PCI_MSI_PENDING_32: u16 = 16;
///
pub const PCI_MSI_PENDING_64: u16 = 20;
///
pub const PCI_PCIX_COMMAND: u16 = 2;
///
pub const PCI_PCIX_COMMAND_DPERE: u16 = 0x0001;
///
pub const PCI_PCIX_COMMAND_ERO: u16 = 0x0002;
///
pub const PCI_PCIX_COMMAND_MAX_MEM_READ_BYTE_COUNT: u16 = 0x000c;
///
pub const PCI_PCIX_COMMAND_MAX_OUTSTANDING_SPLIT_TRANS: u16 = 0x0070;
///
pub const PCI_PCIX_COMMAND_RESERVED: u16 = 0xf80;
///
pub const PCI_PCIX_STATUS: u16 = 4;
///
pub const PCI_PCIX_SIZEOF: u16 = 4;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS: u16 = 2;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_64BIT: u16 = 0x0001;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_133MHZ: u16 = 0x0002;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_SC_DISCARDED: u16 = 0x0004;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_UNEXPECTED_SC: u16 = 0x0008;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_SC_OVERRUN: u16 = 0x0010;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_SPLIT_REQUEST_DELAYED: u16 = 0x0020;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_CLOCK_FREQ: u16 = 0x01c0;
///
pub const PCI_PCIX_BRIDGE_SEC_STATUS_RESERVED: u16 = 0xfe00;
///
pub const PCI_PCIX_BRIDGE_STATUS: u16 = 4;
///
pub const PCI_PCIX_BRIDGE_UPSTREAM_SPLIT_TRANS_CTRL: u16 = 8;
///
pub const PCI_PCIX_BRIDGE_DOWNSTREAM_SPLIT_TRANS_CTRL: u16 = 12;
///
pub const PCI_PCIX_BRIDGE_SIZEOF: u16 = 12;
///
pub const PCI_HT_CMD: u16 = 2;
///
pub const PCI_HT_CMD_TYP_HI: u16 = 0xe000;
///
pub const PCI_HT_CMD_TYP_HI_PRI: u16 = 0x0000;
///
pub const PCI_HT_CMD_TYP_HI_SEC: u16 = 0x2000;
///
pub const PCI_HT_CMD_TYP: u16 = 0xf800;
///
pub const PCI_HT_CMD_TYP_SW: u16 = 0x4000;
///
pub const PCI_HT_CMD_TYP_IDC: u16 = 0x8000;
///
pub const PCI_HT_CMD_TYP_RID: u16 = 0x8800;
///
pub const PCI_HT_CMD_TYP_UIDC: u16 = 0x9000;
///
pub const PCI_HT_CMD_TYP_ECSA: u16 = 0x9800;
///
pub const PCI_HT_CMD_TYP_AM: u16 = 0xa000;
///
pub const PCI_HT_CMD_TYP_MSIM: u16 = 0xa800;
///
pub const PCI_HT_CMD_TYP_DR: u16 = 0xb000;
///
pub const PCI_HT_CMD_TYP_VCS: u16 = 0xb800;
///
pub const PCI_HT_CMD_TYP_RM: u16 = 0xc000;
///
pub const PCI_HT_CMD_TYP_X86: u16 = 0xc800;
///
pub const PCI_HT_LCTR_CFLE: u16 = 0x0002;
///
pub const PCI_HT_LCTR_CST: u16 = 0x0004;
///
pub const PCI_HT_LCTR_CFE: u16 = 0x0008;
///
pub const PCI_HT_LCTR_LKFAIL: u16 = 0x0010;
///
pub const PCI_HT_LCTR_INIT: u16 = 0x0020;
///
pub const PCI_HT_LCTR_EOC: u16 = 0x0040;
///
pub const PCI_HT_LCTR_TXO: u16 = 0x0080;
///
pub const PCI_HT_LCTR_CRCERR: u16 = 0x0f00;
///
pub const PCI_HT_LCTR_ISOCEN: u16 = 0x1000;
///
pub const PCI_HT_LCTR_LSEN: u16 = 0x2000;
///
pub const PCI_HT_LCTR_EXTCTL: u16 = 0x4000;
///
pub const PCI_HT_LCTR_64B: u16 = 0x8000;
///
pub const PCI_HT_LCNF_MLWI: u16 = 0x0007;
///
pub const PCI_HT_LCNF_LW_8B: u8 = 0x0;
///
pub const PCI_HT_LCNF_LW_16B: u8 = 0x1;
///
pub const PCI_HT_LCNF_LW_32B: u8 = 0x3;
///
pub const PCI_HT_LCNF_LW_2B: u8 = 0x4;
///
pub const PCI_HT_LCNF_LW_4B: u8 = 0x5;
///
pub const PCI_HT_LCNF_LW_NC: u8 = 0x7;
///
pub const PCI_HT_LCNF_DFI: u16 = 0x0008;
///
pub const PCI_HT_LCNF_MLWO: u16 = 0x0070;
///
pub const PCI_HT_LCNF_DFO: u16 = 0x0080;
///
pub const PCI_HT_LCNF_LWI: u16 = 0x0700;
///
pub const PCI_HT_LCNF_DFIE: u16 = 0x0800;
///
pub const PCI_HT_LCNF_LWO: u16 = 0x7000;
///
pub const PCI_HT_LCNF_DFOE: u16 = 0x8000;
///
pub const PCI_HT_RID_MIN: u8 = 0x1f;
///
pub const PCI_HT_RID_MAJ: u8 = 0xe0;
///
pub const PCI_HT_LFRER_FREQ: u8 = 0x0f;
///
pub const PCI_HT_LFRER_200: u8 = 0x00;
///
pub const PCI_HT_LFRER_300: u8 = 0x01;
///
pub const PCI_HT_LFRER_400: u8 = 0x02;
///
pub const PCI_HT_LFRER_500: u8 = 0x03;
///
pub const PCI_HT_LFRER_600: u8 = 0x04;
///
pub const PCI_HT_LFRER_800: u8 = 0x05;
///
pub const PCI_HT_LFRER_1000: u8 = 0x06;
///
pub const PCI_HT_LFRER_1200: u8 = 0x07;
///
pub const PCI_HT_LFRER_1400: u8 = 0x08;
///
pub const PCI_HT_LFRER_1600: u8 = 0x09;
///
pub const PCI_HT_LFRER_VEND: u8 = 0x0f;
///
pub const PCI_HT_LFRER_ERR: u8 = 0xf0;
///
pub const PCI_HT_LFRER_PROT: u8 = 0x10;
///
pub const PCI_HT_LFRER_OV: u8 = 0x20;
///
pub const PCI_HT_LFRER_EOC: u8 = 0x40;
///
pub const PCI_HT_LFRER_CTLT: u8 = 0x80;
///
pub const PCI_HT_LFCAP_200: u16 = 0x0001;
///
pub const PCI_HT_LFCAP_300: u16 = 0x0002;
///
pub const PCI_HT_LFCAP_400: u16 = 0x0004;
///
pub const PCI_HT_LFCAP_500: u16 = 0x0008;
///
pub const PCI_HT_LFCAP_600: u16 = 0x0010;
///
pub const PCI_HT_LFCAP_800: u16 = 0x0020;
///
pub const PCI_HT_LFCAP_1000: u16 = 0x0040;
///
pub const PCI_HT_LFCAP_1200: u16 = 0x0080;
///
pub const PCI_HT_LFCAP_1400: u16 = 0x0100;
///
pub const PCI_HT_LFCAP_1600: u16 = 0x0200;
///
pub const PCI_HT_LFCAP_VEND: u16 = 0x8000;
///
pub const PCI_HT_FTR_ISOCFC: u16 = 0x0001;
///
pub const PCI_HT_FTR_LDTSTOP: u16 = 0x0002;
///
pub const PCI_HT_FTR_CRCTM: u16 = 0x0004;
///
pub const PCI_HT_FTR_ECTLT: u16 = 0x0008;
///
pub const PCI_HT_FTR_64BA: u16 = 0x0010;
///
pub const PCI_HT_FTR_UIDRD: u16 = 0x0020;
///
pub const PCI_HT_EH_PFLE: u16 = 0x0001;
///
pub const PCI_HT_EH_OFLE: u16 = 0x0002;
///
pub const PCI_HT_EH_PFE: u16 = 0x0004;
///
pub const PCI_HT_EH_OFE: u16 = 0x0008;
///
pub const PCI_HT_EH_EOCFE: u16 = 0x0010;
///
pub const PCI_HT_EH_RFE: u16 = 0x0020;
///
pub const PCI_HT_EH_CRCFE: u16 = 0x0040;
///
pub const PCI_HT_EH_SERRFE: u16 = 0x0080;
///
pub const PCI_HT_EH_CF: u16 = 0x0100;
///
pub const PCI_HT_EH_RE: u16 = 0x0200;
///
pub const PCI_HT_EH_PNFE: u16 = 0x0400;
///
pub const PCI_HT_EH_ONFE: u16 = 0x0800;
///
pub const PCI_HT_EH_EOCNFE: u16 = 0x1000;
///
pub const PCI_HT_EH_RNFE: u16 = 0x2000;
///
pub const PCI_HT_EH_CRCNFE: u16 = 0x4000;
///
pub const PCI_HT_EH_SERRNFE: u16 = 0x8000;
///
pub const PCI_HT_PRI_CMD: u16 = 2;
///
pub const PCI_HT_PRI_CMD_BUID: u16 = 0x001f;
///
pub const PCI_HT_PRI_CMD_UC: u16 = 0x03e0;
///
pub const PCI_HT_PRI_CMD_MH: u16 = 0x0400;
///
pub const PCI_HT_PRI_CMD_DD: u16 = 0x0800;
///
pub const PCI_HT_PRI_CMD_DUL: u16 = 0x1000;
///
pub const PCI_HT_PRI_LCTR0: u16 = 4;
///
pub const PCI_HT_PRI_LCNF0: u16 = 6;
///
pub const PCI_HT_PRI_LCTR1: u16 = 8;
///
pub const PCI_HT_PRI_LCNF1: u16 = 10;
///
pub const PCI_HT_PRI_RID: u16 = 12;
///
pub const PCI_HT_PRI_LFRER0: u16 = 13;
///
pub const PCI_HT_PRI_LFCAP0: u16 = 14;
///
pub const PCI_HT_PRI_FTR: u16 = 16;
///
pub const PCI_HT_PRI_LFRER1: u16 = 17;
///
pub const PCI_HT_PRI_LFCAP1: u16 = 18;
///
pub const PCI_HT_PRI_ES: u16 = 20;
///
pub const PCI_HT_PRI_EH: u16 = 22;
///
pub const PCI_HT_PRI_MBU: u16 = 24;
///
pub const PCI_HT_PRI_MLU: u16 = 25;
///
pub const PCI_HT_PRI_BN: u16 = 26;
///
pub const PCI_HT_PRI_SIZEOF: u16 = 28;
///
pub const PCI_HT_SEC_CMD: u16 = 2;
///
pub const PCI_HT_SEC_CMD_WR: u16 = 0x0001;
///
pub const PCI_HT_SEC_CMD_DE: u16 = 0x0002;
///
pub const PCI_HT_SEC_CMD_DN: u16 = 0x007c;
///
pub const PCI_HT_SEC_CMD_CS: u16 = 0x0080;
///
pub const PCI_HT_SEC_CMD_HH: u16 = 0x0100;
///
pub const PCI_HT_SEC_CMD_AS: u16 = 0x0400;
///
pub const PCI_HT_SEC_CMD_HIECE: u16 = 0x0800;
///
pub const PCI_HT_SEC_CMD_DUL: u16 = 0x1000;
///
pub const PCI_HT_SEC_LCTR: u16 = 4;
///
pub const PCI_HT_SEC_LCNF: u16 = 6;
///
pub const PCI_HT_SEC_RID: u16 = 8;
///
pub const PCI_HT_SEC_LFRER: u16 = 9;
///
pub const PCI_HT_SEC_LFCAP: u16 = 10;
///
pub const PCI_HT_SEC_FTR: u16 = 12;
///
pub const PCI_HT_SEC_FTR_EXTRS: u16 = 0x0100;
///
pub const PCI_HT_SEC_FTR_UCNFE: u16 = 0x0200;
///
pub const PCI_HT_SEC_ES: u16 = 16;
///
pub const PCI_HT_SEC_EH: u16 = 18;
///
pub const PCI_HT_SEC_MBU: u16 = 20;
///
pub const PCI_HT_SEC_MLU: u16 = 21;
///
pub const PCI_HT_SEC_SIZEOF: u16 = 24;
///
pub const PCI_HT_SW_CMD: u16 = 2;
///
pub const PCI_HT_SW_CMD_VIBERR: u16 = 0x0080;
///
pub const PCI_HT_SW_CMD_VIBFL: u16 = 0x0100;
///
pub const PCI_HT_SW_CMD_VIBFT: u16 = 0x0200;
///
pub const PCI_HT_SW_CMD_VIBNFT: u16 = 0x0400;
///
pub const PCI_HT_SW_PMASK: u16 = 4;
///
pub const PCI_HT_SW_SWINF: u16 = 8;
///
pub const PCI_HT_SW_PCD: u16 = 12;
///
pub const PCI_HT_SW_BLRD: u16 = 16;
///
pub const PCI_HT_SW_SBD: u16 = 20;
///
pub const PCI_HT_SW_SIZEOF: u16 = 24;
///
pub const PCI_HT_SW_PC_PCR: u8 = 0x0;
///
pub const PCI_HT_SW_PC_NPCR: u8 = 0x1;
///
pub const PCI_HT_SW_PC_RCR: u8 = 0x2;
///
pub const PCI_HT_SW_PC_PDWR: u8 = 0x3;
///
pub const PCI_HT_SW_PC_NPDWR: u8 = 0x4;
///
pub const PCI_HT_SW_PC_RDWR: u8 = 0x5;
///
pub const PCI_HT_SW_PC_PCT: u8 = 0x6;
///
pub const PCI_HT_SW_PC_NPCT: u8 = 0x7;
///
pub const PCI_HT_SW_PC_RCT: u8 = 0x8;
///
pub const PCI_HT_SW_PC_PDWT: u8 = 0x9;
///
pub const PCI_HT_SW_PC_NPDWT: u8 = 0xa;
///
pub const PCI_HT_SW_PC_RDWT: u8 = 0xb;
///
pub const PCI_HT_SW_BLR_BASE0_LO: u8 = 0x0;
///
pub const PCI_HT_SW_BLR_BASE0_HI: u8 = 0x1;
///
pub const PCI_HT_SW_BLR_LIM0_LO: u8 = 0x2;
///
pub const PCI_HT_SW_BLR_LIM0_HI: u8 = 0x3;
///
pub const PCI_HT_SW_SB_LO: u8 = 0x0;
///
pub const PCI_HT_SW_S0_HI: u8 = 0x1;
///
pub const PCI_HT_IDC_IDX: u16 = 2;
///
pub const PCI_HT_IDC_DATA: u16 = 4;
///
pub const PCI_HT_IDC_SIZEOF: u16 = 8;
///
pub const PCI_HT_IDC_IDX_LINT: u8 = 0x01;
///
pub const PCI_HT_IDC_IDX_IDR: u8 = 0x10;
///
pub const PCI_HT_RID_RID: u16 = 2;
///
pub const PCI_HT_RID_SIZEOF: u16 = 4;
///
pub const PCI_HT_UIDC_CS: u16 = 4;
///
pub const PCI_HT_UIDC_CE: u16 = 8;
///
pub const PCI_HT_UIDC_SIZEOF: u16 = 12;
///
pub const PCI_HT_ECSA_ADDR: u16 = 4;
///
pub const PCI_HT_ECSA_DATA: u16 = 8;
///
pub const PCI_HT_ECSA_SIZEOF: u16 = 12;
///
pub const PCI_HT_AM_CMD: u16 = 2;
///
pub const PCI_HT_AM_CMD_NDMA: u16 = 0x000f;
///
pub const PCI_HT_AM_CMD_IOSIZ: u16 = 0x01f0;
///
pub const PCI_HT_AM_CMD_MT: u16 = 0x0600;
///
pub const PCI_HT_AM_CMD_MT_40B: u16 = 0x0000;
///
pub const PCI_HT_AM_CMD_MT_64B: u16 = 0x0200;
///
pub const PCI_HT_AM_SBW_CTR_COMP: u8 = 0x1;
///
pub const PCI_HT_AM_SBW_CTR_NCOH: u8 = 0x2;
///
pub const PCI_HT_AM_SBW_CTR_ISOC: u8 = 0x4;
///
pub const PCI_HT_AM_SBW_CTR_EN: u8 = 0x8;
///
pub const PCI_HT_AM40_SBNPW: u16 = 4;
///
pub const PCI_HT_AM40_SBPW: u16 = 8;
///
pub const PCI_HT_AM40_DMA_PBASE0: u16 = 12;
///
pub const PCI_HT_AM40_DMA_CTR0: u16 = 15;
///
pub const PCI_HT_AM40_DMA_CTR_CTR: u8 = 0xf0;
///
pub const PCI_HT_AM40_DMA_SLIM0: u16 = 16;
///
pub const PCI_HT_AM40_DMA_SBASE0: u16 = 18;
///
pub const PCI_HT_AM40_SIZEOF: u16 = 12;
///
pub const PCI_HT_AM64_IDX: u16 = 4;
///
pub const PCI_HT_AM64_DATA_LO: u16 = 8;
///
pub const PCI_HT_AM64_DATA_HI: u16 = 12;
///
pub const PCI_HT_AM64_SIZEOF: u16 = 16;
///
pub const PCI_HT_AM64_IDX_SBNPW: u8 = 0x00;
///
pub const PCI_HT_AM64_IDX_SBPW: u8 = 0x01;
///
pub const PCI_HT_AM64_IDX_PBNPW: u8 = 0x02;
///
pub const PCI_HT_AM64_IDX_DMAPB0: u8 = 0x04;
///
pub const PCI_HT_AM64_IDX_DMASB0: u8 = 0x05;
///
pub const PCI_HT_AM64_IDX_DMASL0: u8 = 0x06;
///
pub const PCI_HT_MSIM_CMD: u16 = 2;
///
pub const PCI_HT_MSIM_CMD_EN: u16 = 0x0001;
///
pub const PCI_HT_MSIM_CMD_FIXD: u16 = 0x0002;
///
pub const PCI_HT_MSIM_ADDR_LO: u16 = 4;
///
pub const PCI_HT_MSIM_ADDR_HI: u16 = 8;
///
pub const PCI_HT_MSIM_SIZEOF: u16 = 12;
///
pub const PCI_HT_DR_CMD: u16 = 2;
///
pub const PCI_HT_DR_CMD_NDRS: u16 = 0x000f;
///
pub const PCI_HT_DR_CMD_IDX: u16 = 0x01f0;
///
pub const PCI_HT_DR_EN: u16 = 4;
///
pub const PCI_HT_DR_DATA: u16 = 8;
///
pub const PCI_HT_DR_SIZEOF: u16 = 12;
///
pub const PCI_HT_DR_IDX_BASE_LO: u8 = 0x00;
///
pub const PCI_HT_DR_IDX_BASE_HI: u8 = 0x01;
///
pub const PCI_HT_DR_IDX_LIMIT_LO: u8 = 0x02;
///
pub const PCI_HT_DR_IDX_LIMIT_HI: u8 = 0x03;
///
pub const PCI_HT_VCS_SUP: u16 = 4;
///
pub const PCI_HT_VCS_L1EN: u16 = 5;
///
pub const PCI_HT_VCS_L0EN: u16 = 6;
///
pub const PCI_HT_VCS_SBD: u16 = 8;
///
pub const PCI_HT_VCS_SINT: u16 = 9;
///
pub const PCI_HT_VCS_SSUP: u16 = 10;
///
pub const PCI_HT_VCS_SSUP_0: u8 = 0x00;
///
pub const PCI_HT_VCS_SSUP_3: u8 = 0x01;
///
pub const PCI_HT_VCS_SSUP_15: u8 = 0x02;
///
pub const PCI_HT_VCS_NFCBD: u16 = 12;
///
pub const PCI_HT_VCS_NFCINT: u16 = 13;
///
pub const PCI_HT_VCS_SIZEOF: u16 = 16;
///
pub const PCI_HT_RM_CTR0: u16 = 4;
///
pub const PCI_HT_RM_CTR_LRETEN: u8 = 0x01;
///
pub const PCI_HT_RM_CTR_FSER: u8 = 0x02;
///
pub const PCI_HT_RM_CTR_ROLNEN: u8 = 0x04;
///
pub const PCI_HT_RM_CTR_FSS: u8 = 0x08;
///
pub const PCI_HT_RM_CTR_RETNEN: u8 = 0x10;
///
pub const PCI_HT_RM_CTR_RETFEN: u8 = 0x20;
///
pub const PCI_HT_RM_CTR_AA: u8 = 0xc0;
///
pub const PCI_HT_RM_STS0: u16 = 5;
///
pub const PCI_HT_RM_STS_RETSNT: u8 = 0x01;
///
pub const PCI_HT_RM_STS_CNTROL: u8 = 0x02;
///
pub const PCI_HT_RM_STS_SRCV: u8 = 0x04;
///
pub const PCI_HT_RM_CTR1: u16 = 6;
///
pub const PCI_HT_RM_STS1: u16 = 7;
///
pub const PCI_HT_RM_CNT0: u16 = 8;
///
pub const PCI_HT_RM_CNT1: u16 = 10;
///
pub const PCI_HT_RM_SIZEOF: u16 = 12;
///
pub const PCI_VNDR_LENGTH: u16 = 2;
///
pub const PCI_EXP_FLAGS: u8 = 0x2;
///
pub const PCI_EXP_FLAGS_VERS: u16 = 0x000f;
///
pub const PCI_EXP_FLAGS_TYPE: u16 = 0x00f0;
///
pub const PCI_EXP_TYPE_ENDPOINT: u8 = 0x0;
///
pub const PCI_EXP_TYPE_LEG_END: u8 = 0x1;
///
pub const PCI_EXP_TYPE_ROOT_PORT: u8 = 0x4;
///
pub const PCI_EXP_TYPE_UPSTREAM: u8 = 0x5;
///
pub const PCI_EXP_TYPE_DOWNSTREAM: u8 = 0x6;
///
pub const PCI_EXP_TYPE_PCI_BRIDGE: u8 = 0x7;
///
pub const PCI_EXP_TYPE_PCIE_BRIDGE: u8 = 0x8;
///
pub const PCI_EXP_TYPE_ROOT_INT_EP: u8 = 0x9;
///
pub const PCI_EXP_TYPE_ROOT_EC: u8 = 0xa;
///
pub const PCI_EXP_FLAGS_SLOT: u16 = 0x0100;
///
pub const PCI_EXP_FLAGS_IRQ: u16 = 0x3e00;
///
pub const PCI_EXP_DEVCAP: u8 = 0x4;
///
pub const PCI_EXP_DEVCAP_PAYLOAD: u8 = 0x07;
///
pub const PCI_EXP_DEVCAP_PHANTOM: u8 = 0x18;
///
pub const PCI_EXP_DEVCAP_EXT_TAG: u8 = 0x20;
///
pub const PCI_EXP_DEVCAP_L0S: u16 = 0x1c0;
///
pub const PCI_EXP_DEVCAP_L1: u16 = 0xe00;
///
pub const PCI_EXP_DEVCAP_ATN_BUT: u16 = 0x1000;
///
pub const PCI_EXP_DEVCAP_ATN_IND: u16 = 0x2000;
///
pub const PCI_EXP_DEVCAP_PWR_IND: u16 = 0x4000;
///
pub const PCI_EXP_DEVCAP_RBE: u16 = 0x8000;
///
pub const PCI_EXP_DEVCTL: u8 = 0x8;
///
pub const PCI_EXP_DEVCTL_CERE: u16 = 0x0001;
///
pub const PCI_EXP_DEVCTL_NFERE: u16 = 0x0002;
///
pub const PCI_EXP_DEVCTL_FERE: u16 = 0x0004;
///
pub const PCI_EXP_DEVCTL_URRE: u16 = 0x0008;
///
pub const PCI_EXP_DEVCTL_RELAXED: u16 = 0x0010;
///
pub const PCI_EXP_DEVCTL_PAYLOAD: u16 = 0x00e0;
///
pub const PCI_EXP_DEVCTL_EXT_TAG: u16 = 0x0100;
///
pub const PCI_EXP_DEVCTL_PHANTOM: u16 = 0x0200;
///
pub const PCI_EXP_DEVCTL_AUX_PME: u16 = 0x0400;
///
pub const PCI_EXP_DEVCTL_NOSNOOP: u16 = 0x0800;
///
pub const PCI_EXP_DEVCTL_READRQ: u16 = 0x7000;
///
pub const PCI_EXP_DEVCTL_BCRE: u16 = 0x8000;
///
pub const PCI_EXP_DEVCTL_FLRESET: u16 = 0x8000;
///
pub const PCI_EXP_DEVSTA: u8 = 0xa;
///
pub const PCI_EXP_DEVSTA_CED: u8 = 0x01;
///
pub const PCI_EXP_DEVSTA_NFED: u8 = 0x02;
///
pub const PCI_EXP_DEVSTA_FED: u8 = 0x04;
///
pub const PCI_EXP_DEVSTA_URD: u8 = 0x08;
///
pub const PCI_EXP_DEVSTA_AUXPD: u8 = 0x10;
///
pub const PCI_EXP_DEVSTA_TRPND: u8 = 0x20;
///
pub const PCI_EXP_LNKCAP: u8 = 0xc;
///
pub const PCI_EXP_LNKCTL: u8 = 0x10;
///
pub const PCI_EXP_LNKCTL_ASPM: u16 = 0x0003;
///
pub const PCI_EXP_LNKCTL_RCB: u16 = 0x0008;
///
pub const PCI_EXP_LNKCTL_DISABLE: u16 = 0x0010;
///
pub const PCI_EXP_LNKCTL_RETRAIN: u16 = 0x0020;
///
pub const PCI_EXP_LNKCTL_CLOCK: u16 = 0x0040;
///
pub const PCI_EXP_LNKCTL_XSYNCH: u16 = 0x0080;
///
pub const PCI_EXP_LNKCTL_CLOCKPM: u16 = 0x0100;
///
pub const PCI_EXP_LNKCTL_HWAUTWD: u16 = 0x0200;
///
pub const PCI_EXP_LNKCTL_BWMIE: u16 = 0x0400;
///
pub const PCI_EXP_LNKCTL_AUTBWIE: u16 = 0x0800;
///
pub const PCI_EXP_LNKSTA: u8 = 0x12;
///
pub const PCI_EXP_LNKSTA_SPEED: u16 = 0x000f;
///
pub const PCI_EXP_LNKSTA_WIDTH: u16 = 0x03f0;
///
pub const PCI_EXP_LNKSTA_TR_ERR: u16 = 0x0400;
///
pub const PCI_EXP_LNKSTA_TRAIN: u16 = 0x0800;
///
pub const PCI_EXP_LNKSTA_SL_CLK: u16 = 0x1000;
///
pub const PCI_EXP_LNKSTA_DL_ACT: u16 = 0x2000;
///
pub const PCI_EXP_LNKSTA_BWMGMT: u16 = 0x4000;
///
pub const PCI_EXP_LNKSTA_AUTBW: u16 = 0x8000;
///
pub const PCI_EXP_SLTCAP: u8 = 0x14;
///
pub const PCI_EXP_SLTCAP_ATNB: u16 = 0x0001;
///
pub const PCI_EXP_SLTCAP_PWRC: u16 = 0x0002;
///
pub const PCI_EXP_SLTCAP_MRL: u16 = 0x0004;
///
pub const PCI_EXP_SLTCAP_ATNI: u16 = 0x0008;
///
pub const PCI_EXP_SLTCAP_PWRI: u16 = 0x0010;
///
pub const PCI_EXP_SLTCAP_HPS: u16 = 0x0020;
///
pub const PCI_EXP_SLTCAP_HPC: u16 = 0x0040;
///
pub const PCI_EXP_SLTCTL: u8 = 0x18;
///
pub const PCI_EXP_SLTCTL_ATNB: u16 = 0x0001;
///
pub const PCI_EXP_SLTCTL_PWRF: u16 = 0x0002;
///
pub const PCI_EXP_SLTCTL_MRLS: u16 = 0x0004;
///
pub const PCI_EXP_SLTCTL_PRSD: u16 = 0x0008;
///
pub const PCI_EXP_SLTCTL_CMDC: u16 = 0x0010;
///
pub const PCI_EXP_SLTCTL_HPIE: u16 = 0x0020;
///
pub const PCI_EXP_SLTCTL_ATNI: u16 = 0x00c0;
///
pub const PCI_EXP_SLTCTL_PWRI: u16 = 0x0300;
///
pub const PCI_EXP_SLTCTL_PWRC: u16 = 0x0400;
///
pub const PCI_EXP_SLTCTL_INTERLOCK: u16 = 0x0800;
///
pub const PCI_EXP_SLTCTL_LLCHG: u16 = 0x1000;
///
pub const PCI_EXP_SLTSTA: u8 = 0x1a;
///
pub const PCI_EXP_SLTSTA_ATNB: u16 = 0x0001;
///
pub const PCI_EXP_SLTSTA_PWRF: u16 = 0x0002;
///
pub const PCI_EXP_SLTSTA_MRLS: u16 = 0x0004;
///
pub const PCI_EXP_SLTSTA_PRSD: u16 = 0x0008;
///
pub const PCI_EXP_SLTSTA_CMDC: u16 = 0x0010;
///
pub const PCI_EXP_SLTSTA_MRL_ST: u16 = 0x0020;
///
pub const PCI_EXP_SLTSTA_PRES: u16 = 0x0040;
///
pub const PCI_EXP_SLTSTA_INTERLOCK: u16 = 0x0080;
///
pub const PCI_EXP_SLTSTA_LLCHG: u16 = 0x0100;
///
pub const PCI_EXP_RTCTL: u8 = 0x1c;
///
pub const PCI_EXP_RTCTL_SECEE: u16 = 0x0001;
///
pub const PCI_EXP_RTCTL_SENFEE: u16 = 0x0002;
///
pub const PCI_EXP_RTCTL_SEFEE: u16 = 0x0004;
///
pub const PCI_EXP_RTCTL_PMEIE: u16 = 0x0008;
///
pub const PCI_EXP_RTCTL_CRSVIS: u16 = 0x0010;
///
pub const PCI_EXP_RTCAP: u8 = 0x1e;
///
pub const PCI_EXP_RTCAP_CRSVIS: u16 = 0x0001;
///
pub const PCI_EXP_RTSTA: u8 = 0x20;
///
pub const PCI_EXP_DEVCAP2: u8 = 0x24;
///
pub const PCI_EXP_DEVCAP2_TIMEOUT_DIS: u16 = 0x0010;
///
pub const PCI_EXP_DEVCAP2_ARI: u16 = 0x0020;
///
pub const PCI_EXP_DEVCAP2_ATOMICOP_ROUTING: u16 = 0x0040;
///
pub const PCI_EXP_DEVCAP2_32BIT_ATOMICOP_COMP: u16 = 0x0080;
///
pub const PCI_EXP_DEVCAP2_64BIT_ATOMICOP_COMP: u16 = 0x0100;
///
pub const PCI_EXP_DEVCAP2_128BIT_CAS_COMP: u16 = 0x0200;
///
pub const PCI_EXP_DEVCAP2_NROPRPRP: u16 = 0x0400;
///
pub const PCI_EXP_DEVCAP2_LTR: u16 = 0x0800;
///
pub const PCI_EXP_DEVCTL2: u8 = 0x28;
///
pub const PCI_EXP_DEVCTL2_TIMEOUT_DIS: u16 = 0x0010;
///
pub const PCI_EXP_DEVCTL2_ARI: u16 = 0x0020;
///
pub const PCI_EXP_DEVCTL2_ATOMICOP_REQUESTER_EN: u16 = 0x0040;
///
pub const PCI_EXP_DEVCTL2_ATOMICOP_EGRESS_BLOCK: u16 = 0x0080;
///
pub const PCI_EXP_DEVCTL2_IDO_REQ_EN: u16 = 0x0100;
///
pub const PCI_EXP_DEVCTL2_IDO_CMP_EN: u16 = 0x0200;
///
pub const PCI_EXP_DEVCTL2_LTR: u16 = 0x0400;
///
pub const PCI_EXP_DEVCTL2_EPR_REQ: u16 = 0x0800;
///
pub const PCI_EXP_DEVCTL2_10BIT_TAG_REQ: u16 = 0x1000;
///
pub const PCI_EXP_DEVCTL2_EE_TLP_BLK: u16 = 0x8000;
///
pub const PCI_EXP_DEVSTA2: u8 = 0x2a;
///
pub const PCI_EXP_LNKCAP2: u8 = 0x2c;
///
pub const PCI_EXP_LNKCTL2: u8 = 0x30;
///
pub const PCI_EXP_LNKCTL2_CMPLNC: u16 = 0x0010;
///
pub const PCI_EXP_LNKCTL2_SPEED_DIS: u16 = 0x0020;
///
pub const PCI_EXP_LNKCTL2_MOD_CMPLNC: u16 = 0x0400;
///
pub const PCI_EXP_LNKCTL2_CMPLNC_SOS: u16 = 0x0800;
///
pub const PCI_EXP_LNKSTA2: u8 = 0x32;
///
pub const PCI_EXP_LINKSTA2_EQU_COMP: u8 = 0x02;
///
pub const PCI_EXP_LINKSTA2_EQU_PHASE1: u8 = 0x04;
///
pub const PCI_EXP_LINKSTA2_EQU_PHASE2: u8 = 0x08;
///
pub const PCI_EXP_LINKSTA2_EQU_PHASE3: u8 = 0x10;
///
pub const PCI_EXP_LINKSTA2_EQU_REQ: u8 = 0x20;
///
pub const PCI_EXP_LINKSTA2_RETIMER: u16 = 0x0040;
///
pub const PCI_EXP_LINKSTA2_2RETIMERS: u16 = 0x0080;
///
pub const PCI_EXP_LINKSTA2_DRS_RCVD: u16 = 0x8000;
///
pub const PCI_EXP_SLTCAP2: u8 = 0x34;
///
pub const PCI_EXP_SLTCTL2: u8 = 0x38;
///
pub const PCI_EXP_SLTSTA2: u8 = 0x3a;
///
pub const PCI_MSIX_ENABLE: u16 = 0x8000;
///
pub const PCI_MSIX_MASK: u16 = 0x4000;
///
pub const PCI_MSIX_TABSIZE: u16 = 0x07ff;
///
pub const PCI_MSIX_TABLE: u16 = 4;
///
pub const PCI_MSIX_PBA: u16 = 8;
///
pub const PCI_MSIX_BIR: u8 = 0x7;
///
pub const PCI_SSVID_VENDOR: u16 = 4;
///
pub const PCI_SSVID_DEVICE: u16 = 6;
///
pub const PCI_AF_CAP: u16 = 3;
///
pub const PCI_AF_CAP_TP: u8 = 0x01;
///
pub const PCI_AF_CAP_FLR: u8 = 0x02;
///
pub const PCI_AF_CTRL: u16 = 4;
///
pub const PCI_AF_CTRL_FLR: u8 = 0x01;
///
pub const PCI_AF_STATUS: u16 = 5;
///
pub const PCI_AF_STATUS_TP: u8 = 0x01;
///
pub const PCI_SATA_HBA_BARS: u16 = 4;
///
pub const PCI_SATA_HBA_REG0: u16 = 8;
///
pub const PCI_EA_CAP_TYPE1_SECONDARY: u16 = 4;
///
pub const PCI_EA_CAP_TYPE1_SUBORDINATE: u16 = 5;
///
pub const PCI_ERR_UNCOR_STATUS: u16 = 4;
///
pub const PCI_ERR_UNCOR_MASK: u16 = 8;
///
pub const PCI_ERR_UNCOR_SEVER: u16 = 12;
///
pub const PCI_ERR_COR_STATUS: u16 = 16;
///
pub const PCI_ERR_COR_MASK: u16 = 20;
///
pub const PCI_ERR_CAP: u16 = 24;
///
pub const PCI_ERR_HEADER_LOG: u16 = 28;
///
pub const PCI_ERR_ROOT_COMMAND: u16 = 44;
///
pub const PCI_ERR_ROOT_STATUS: u16 = 48;
///
pub const PCI_ERR_ROOT_COR_SRC: u16 = 52;
///
pub const PCI_ERR_ROOT_SRC: u16 = 54;
///
pub const PCI_VC_PORT_REG1: u16 = 4;
///
pub const PCI_VC_PORT_REG2: u16 = 8;
///
pub const PCI_VC_PORT_CTRL: u16 = 12;
///
pub const PCI_VC_PORT_STATUS: u16 = 14;
///
pub const PCI_VC_RES_CAP: u16 = 16;
///
pub const PCI_VC_RES_CTRL: u16 = 20;
///
pub const PCI_VC_RES_STATUS: u16 = 26;
///
pub const PCI_PWR_DSR: u16 = 4;
///
pub const PCI_PWR_DATA: u16 = 8;
///
pub const PCI_PWR_CAP: u16 = 12;
///
pub const PCI_RCLINK_ESD: u16 = 4;
///
pub const PCI_RCLINK_LINK1: u16 = 16;
///
pub const PCI_RCLINK_LINK_DESC: u16 = 0;
///
pub const PCI_RCLINK_LINK_ADDR: u16 = 8;
///
pub const PCI_RCLINK_LINK_SIZE: u16 = 16;
///
pub const PCI_RCEC_BUSN_REG_VER: u8 = 0x02;
///
pub const PCI_RCEC_RCIEP_BMAP: u16 = 0x0004;
///
pub const PCI_RCEC_BUSN_REG: u16 = 0x0008;
///
pub const PCI_EVNDR_HEADER: u16 = 4;
///
pub const PCI_EVNDR_REGISTERS: u16 = 8;
///
pub const PCI_DVSEC_HEADER1: u16 = 4;
///
pub const PCI_DVSEC_HEADER2: u16 = 8;
///
pub const PCI_DVSEC_VENDOR_ID_CXL: u16 = 0x1e98;
///
pub const PCI_DVSEC_ID_CXL: u16 = 0;
///
pub const PCI_CXL_DEV_LEN: u8 = 0x38;
///
pub const PCI_CXL_DEV_LEN_REV2: u8 = 0x3c;
///
pub const PCI_CXL_DEV_CAP: u8 = 0x0a;
///
pub const PCI_CXL_DEV_CAP_CACHE: u16 = 0x0001;
///
pub const PCI_CXL_DEV_CAP_IO: u16 = 0x0002;
///
pub const PCI_CXL_DEV_CAP_MEM: u16 = 0x0004;
///
pub const PCI_CXL_DEV_CAP_MEM_HWINIT: u16 = 0x0008;
///
pub const PCI_CXL_DEV_CAP_VIRAL: u16 = 0x4000;
///
pub const PCI_CXL_DEV_CTRL: u8 = 0x0c;
///
pub const PCI_CXL_DEV_CTRL_CACHE: u16 = 0x0001;
///
pub const PCI_CXL_DEV_CTRL_IO: u16 = 0x0002;
///
pub const PCI_CXL_DEV_CTRL_MEM: u16 = 0x0004;
///
pub const PCI_CXL_DEV_CTRL_CACHE_CLN: u16 = 0x0800;
///
pub const PCI_CXL_DEV_CTRL_VIRAL: u16 = 0x4000;
///
pub const PCI_CXL_DEV_STATUS: u8 = 0x0e;
///
pub const PCI_CXL_DEV_STATUS_VIRAL: u16 = 0x4000;
///
pub const PCI_CXL_DEV_CTRL2: u8 = 0x10;
///
pub const PCI_CXL_DEV_CTRL2_DISABLE_CACHING: u16 = 0x0001;
///
pub const PCI_CXL_DEV_CTRL2_INIT_WB_INVAL: u16 = 0x0002;
///
pub const PCI_CXL_DEV_CTRL2_INIT_CXL_RST: u16 = 0x0003;
///
pub const PCI_CXL_DEV_CTRL2_INIT_CXL_RST_CLR_EN: u16 = 0x0004;
///
pub const PCI_CXL_DEV_CTRL2_INIT_CXL_HDM_STATE_HOTRST: u16 = 0x0005;
///
pub const PCI_CXL_DEV_STATUS2: u8 = 0x12;
///
pub const PCI_CXL_DEV_STATUS_CACHE_INV: u16 = 0x0001;
///
pub const PCI_CXL_DEV_STATUS_RC: u16 = 0x0002;
///
pub const PCI_CXL_DEV_STATUS_RE: u16 = 0x0004;
///
pub const PCI_CXL_DEV_STATUS_PMC: u16 = 0x8000;
///
pub const PCI_CXL_DEV_CAP2: u8 = 0x16;
///
pub const PCI_CXL_DEV_CAP2_CACHE_UNK: u16 = 0x0000;
///
pub const PCI_CXL_DEV_CAP2_CACHE_64K: u16 = 0x0001;
///
pub const PCI_CXL_DEV_CAP2_CACHE_1M: u16 = 0x0002;
///
pub const PCI_CXL_DEV_RANGE1_SIZE_HI: u8 = 0x18;
///
pub const PCI_CXL_DEV_RANGE1_SIZE_LO: u8 = 0x1c;
///
pub const PCI_CXL_RANGE_VALID: u16 = 0x0001;
///
pub const PCI_CXL_RANGE_ACTIVE: u16 = 0x0002;
///
pub const PCI_CXL_DEV_RANGE1_BASE_HI: u8 = 0x20;
///
pub const PCI_CXL_DEV_RANGE1_BASE_LO: u8 = 0x24;
///
pub const PCI_CXL_DEV_RANGE2_SIZE_HI: u8 = 0x28;
///
pub const PCI_CXL_DEV_RANGE2_SIZE_LO: u8 = 0x2c;
///
pub const PCI_CXL_DEV_RANGE2_BASE_HI: u8 = 0x30;
///
pub const PCI_CXL_DEV_RANGE2_BASE_LO: u8 = 0x34;
///
pub const PCI_CXL_DEV_CAP3: u8 = 0x38;
///
pub const PCI_CXL_DEV_CAP3_HDM_STATE_RST_COLD: u16 = 0x0001;
///
pub const PCI_CXL_DEV_CAP3_HDM_STATE_RST_WARM: u16 = 0x0002;
///
pub const PCI_CXL_DEV_CAP3_HDM_STATE_RST_HOT: u16 = 0x0003;
///
pub const PCI_CXL_DEV_CAP3_HDM_STATE_RST_HOT_CFG: u16 = 0x0004;
///
pub const PCI_CXL_PORT_EXT_LEN: u8 = 0x28;
///
pub const PCI_CXL_PORT_EXT_STATUS: u8 = 0x0a;
///
pub const PCI_CXL_PORT_PM_INIT_COMPLETE: u8 = 0x1;
///
pub const PCI_CXL_PORT_CTRL: u8 = 0x0c;
///
pub const PCI_CXL_PORT_UNMASK_SBR: u16 = 0x0001;
///
pub const PCI_CXL_PORT_UNMASK_LINK: u16 = 0x0002;
///
pub const PCI_CXL_PORT_ALT_MEMORY: u16 = 0x0004;
///
pub const PCI_CXL_PORT_ALT_BME: u16 = 0x0008;
///
pub const PCI_CXL_PORT_VIRAL_EN: u16 = 0x4000;
///
pub const PCI_CXL_PORT_ALT_BUS_BASE: u8 = 0xe;
///
pub const PCI_CXL_PORT_ALT_BUS_LIMIT: u8 = 0xf;
///
pub const PCI_CXL_PORT_ALT_MEM_BASE: u8 = 0x10;
///
pub const PCI_CXL_PORT_ALT_MEM_LIMIT: u8 = 0x12;
///
pub const PCI_CXL_RL_BLOCK1_LO: u8 = 0x0c;
///
pub const PCI_CXL_GPF_DEV_LEN: u8 = 0x10;
///
pub const PCI_CXL_GPF_DEV_PHASE2_DUR: u8 = 0x0a;
///
pub const PCI_CXL_GPF_DEV_PHASE2_POW: u8 = 0x0c;
///
pub const PCI_CXL_GPF_DEV_1US: u8 = 0x0;
///
pub const PCI_CXL_GPF_DEV_10US: u8 = 0x1;
///
pub const PCI_CXL_GPF_DEV_100US: u8 = 0x2;
///
pub const PCI_CXL_GPF_DEV_1MS: u8 = 0x3;
///
pub const PCI_CXL_GPF_DEV_10MS: u8 = 0x4;
///
pub const PCI_CXL_GPF_DEV_100MS: u8 = 0x5;
///
pub const PCI_CXL_GPF_DEV_1S: u8 = 0x6;
///
pub const PCI_CXL_GPF_DEV_10S: u8 = 0x7;
///
pub const PCI_CXL_GPF_PORT_LEN: u8 = 0x10;
///
pub const PCI_CXL_GPF_PORT_PHASE1_CTRL: u8 = 0x0c;
///
pub const PCI_CXL_GPF_PORT_PHASE2_CTRL: u8 = 0x0e;
///
pub const PCI_CXL_GPF_PORT_1US: u8 = 0x0;
///
pub const PCI_CXL_GPF_PORT_10US: u8 = 0x1;
///
pub const PCI_CXL_GPF_PORT_100US: u8 = 0x2;
///
pub const PCI_CXL_GPF_PORT_1MS: u8 = 0x3;
///
pub const PCI_CXL_GPF_PORT_10MS: u8 = 0x4;
///
pub const PCI_CXL_GPF_PORT_100MS: u8 = 0x5;
///
pub const PCI_CXL_GPF_PORT_1S: u8 = 0x6;
///
pub const PCI_CXL_GPF_PORT_10S: u8 = 0x7;
///
pub const PCI_CXL_FB_LEN: u8 = 0x20;
///
pub const PCI_CXL_FB_PORT_CAP: u8 = 0x0a;
///
pub const PCI_CXL_FB_CAP_CACHE: u16 = 0x0001;
///
pub const PCI_CXL_FB_CAP_IO: u16 = 0x0002;
///
pub const PCI_CXL_FB_CAP_MEM: u16 = 0x0004;
///
pub const PCI_CXL_FB_CAP_68B_FLIT: u16 = 0x0020;
///
pub const PCI_CXL_FB_CAP_MULT_LOG_DEV: u16 = 0x0040;
///
pub const PCI_CXL_FB_CAP_256B_FLIT: u16 = 0x2000;
///
pub const PCI_CXL_FB_CAP_PBR_FLIT: u16 = 0x4000;
///
pub const PCI_CXL_FB_PORT_CTRL: u8 = 0x0c;
///
pub const PCI_CXL_FB_CTRL_CACHE: u16 = 0x0001;
///
pub const PCI_CXL_FB_CTRL_IO: u16 = 0x0002;
///
pub const PCI_CXL_FB_CTRL_MEM: u16 = 0x0004;
///
pub const PCI_CXL_FB_CTRL_SYNC_HDR_BYP: u16 = 0x0008;
///
pub const PCI_CXL_FB_CTRL_DRFT_BUF: u16 = 0x0010;
///
pub const PCI_CXL_FB_CTRL_68B_FLIT: u16 = 0x0020;
///
pub const PCI_CXL_FB_CTRL_MULT_LOG_DEV: u16 = 0x0040;
///
pub const PCI_CXL_FB_CTRL_RCD: u16 = 0x0080;
///
pub const PCI_CXL_FB_CTRL_RETIMER1: u16 = 0x0100;
///
pub const PCI_CXL_FB_CTRL_RETIMER2: u16 = 0x0200;
///
pub const PCI_CXL_FB_CTRL_256B_FLIT: u16 = 0x2000;
///
pub const PCI_CXL_FB_CTRL_PBR_FLIT: u16 = 0x4000;
///
pub const PCI_CXL_FB_PORT_STATUS: u8 = 0x0e;
///
pub const PCI_CXL_FB_STAT_CACHE: u16 = 0x0001;
///
pub const PCI_CXL_FB_STAT_IO: u16 = 0x0002;
///
pub const PCI_CXL_FB_STAT_MEM: u16 = 0x0004;
///
pub const PCI_CXL_FB_STAT_SYNC_HDR_BYP: u16 = 0x0008;
///
pub const PCI_CXL_FB_STAT_DRFT_BUF: u16 = 0x0010;
///
pub const PCI_CXL_FB_STAT_68B_FLIT: u16 = 0x0020;
///
pub const PCI_CXL_FB_STAT_MULT_LOG_DEV: u16 = 0x0040;
///
pub const PCI_CXL_FB_STAT_256B_FLIT: u16 = 0x2000;
///
pub const PCI_CXL_FB_STAT_PBR_FLIT: u16 = 0x4000;
///
pub const PCI_CXL_FB_MOD_TS_DATA: u8 = 0x10;
///
pub const PCI_CXL_FB_PORT_CAP2: u8 = 0x14;
///
pub const PCI_CXL_FB_CAP2_NOP_HINT: u8 = 0x01;
///
pub const PCI_CXL_FB_PORT_CTRL2: u8 = 0x18;
///
pub const PCI_CXL_FB_CTRL2_NOP_HINT: u8 = 0x01;
///
pub const PCI_CXL_FB_PORT_STATUS2: u8 = 0x1c;
///
pub const PCI_CXL_FB_NEXT_UNSUPPORTED: u8 = 0x20;
///
pub const PCI_CXL_MLD_LEN: u8 = 0x10;
///
pub const PCI_CXL_MLD_NUM_LD: u8 = 0xa;
///
pub const PCI_CXL_MLD_MAX_LD: u8 = 0x10;
///
pub const PCI_CXL_FUN_MAP_LEN: u8 = 0x2c;
///
pub const PCI_CXL_FUN_MAP_REG_0: u8 = 0x0c;
///
pub const PCI_CXL_FUN_MAP_REG_1: u8 = 0x10;
///
pub const PCI_CXL_FUN_MAP_REG_2: u8 = 0x14;
///
pub const PCI_CXL_FUN_MAP_REG_3: u8 = 0x18;
///
pub const PCI_CXL_FUN_MAP_REG_4: u8 = 0x1c;
///
pub const PCI_CXL_FUN_MAP_REG_5: u8 = 0x20;
///
pub const PCI_CXL_FUN_MAP_REG_6: u8 = 0x24;
///
pub const PCI_CXL_FUN_MAP_REG_7: u8 = 0x28;
///
pub const PCI_ACS_CAP: u8 = 0x04;
///
pub const PCI_ACS_CAP_VALID: u16 = 0x0001;
///
pub const PCI_ACS_CAP_BLOCK: u16 = 0x0002;
///
pub const PCI_ACS_CAP_REQ_RED: u16 = 0x0004;
///
pub const PCI_ACS_CAP_CMPLT_RED: u16 = 0x0008;
///
pub const PCI_ACS_CAP_FORWARD: u16 = 0x0010;
///
pub const PCI_ACS_CAP_EGRESS: u16 = 0x0020;
///
pub const PCI_ACS_CAP_TRANS: u16 = 0x0040;
///
pub const PCI_ACS_CTRL: u8 = 0x06;
///
pub const PCI_ACS_CTRL_VALID: u16 = 0x0001;
///
pub const PCI_ACS_CTRL_BLOCK: u16 = 0x0002;
///
pub const PCI_ACS_CTRL_REQ_RED: u16 = 0x0004;
///
pub const PCI_ACS_CTRL_CMPLT_RED: u16 = 0x0008;
///
pub const PCI_ACS_CTRL_FORWARD: u16 = 0x0010;
///
pub const PCI_ACS_CTRL_EGRESS: u16 = 0x0020;
///
pub const PCI_ACS_CTRL_TRANS: u16 = 0x0040;
///
pub const PCI_ACS_EGRESS_CTRL: u8 = 0x08;
///
pub const PCI_ARI_CAP: u8 = 0x04;
///
pub const PCI_ARI_CAP_MFVC: u16 = 0x0001;
///
pub const PCI_ARI_CAP_ACS: u16 = 0x0002;
///
pub const PCI_ARI_CTRL: u8 = 0x06;
///
pub const PCI_ARI_CTRL_MFVC: u16 = 0x0001;
///
pub const PCI_ARI_CTRL_ACS: u16 = 0x0002;
///
pub const PCI_ATS_CAP: u8 = 0x04;
///
pub const PCI_ATS_CTRL: u8 = 0x06;
///
pub const PCI_ATS_CTRL_ENABLE: u16 = 0x8000;
///
pub const PCI_IOV_CAP: u8 = 0x04;
///
pub const PCI_IOV_CTRL: u8 = 0x08;
///
pub const PCI_IOV_CTRL_VFE: u16 = 0x0001;
///
pub const PCI_IOV_CTRL_VFME: u16 = 0x0002;
///
pub const PCI_IOV_CTRL_VFMIE: u16 = 0x0004;
///
pub const PCI_IOV_CTRL_MSE: u16 = 0x0008;
///
pub const PCI_IOV_CTRL_ARI: u16 = 0x0010;
///
pub const PCI_IOV_CTRL_VF_10BIT_TAG_REQ_EN: u16 = 0x0020;
///
pub const PCI_IOV_STATUS: u8 = 0x0a;
///
pub const PCI_IOV_STATUS_MS: u16 = 0x0001;
///
pub const PCI_IOV_INITIALVF: u8 = 0x0c;
///
pub const PCI_IOV_TOTALVF: u8 = 0x0e;
///
pub const PCI_IOV_NUMVF: u8 = 0x10;
///
pub const PCI_IOV_FDL: u8 = 0x12;
///
pub const PCI_IOV_OFFSET: u8 = 0x14;
///
pub const PCI_IOV_STRIDE: u8 = 0x16;
///
pub const PCI_IOV_DID: u8 = 0x1a;
///
pub const PCI_IOV_SUPPS: u8 = 0x1c;
///
pub const PCI_IOV_SYSPS: u8 = 0x20;
///
pub const PCI_IOV_BAR_BASE: u8 = 0x24;
///
pub const PCI_IOV_NUM_BAR: u16 = 6;
///
pub const PCI_IOV_MSAO: u8 = 0x3c;
///
pub const PCI_MCAST_CAP: u8 = 0x04;
///
pub const PCI_MCAST_CAP_ECRC: u16 = 0x8000;
///
pub const PCI_MCAST_CTRL: u8 = 0x06;
///
pub const PCI_MCAST_CTRL_ENABLE: u16 = 0x8000;
///
pub const PCI_MCAST_BAR: u8 = 0x08;
///
pub const PCI_MCAST_RCV: u8 = 0x10;
///
pub const PCI_MCAST_BLOCK: u8 = 0x18;
///
pub const PCI_MCAST_BLOCK_UNTRANS: u8 = 0x20;
///
pub const PCI_MCAST_OVL_BAR: u8 = 0x28;
///
pub const PCI_PRI_CTRL: u8 = 0x04;
///
pub const PCI_PRI_CTRL_ENABLE: u8 = 0x01;
///
pub const PCI_PRI_CTRL_RESET: u8 = 0x02;
///
pub const PCI_PRI_STATUS: u8 = 0x06;
///
pub const PCI_PRI_STATUS_RF: u16 = 0x0001;
///
pub const PCI_PRI_STATUS_UPRGI: u16 = 0x0002;
///
pub const PCI_PRI_STATUS_STOPPED: u16 = 0x0100;
///
pub const PCI_PRI_STATUS_PASID: u16 = 0x8000;
///
pub const PCI_PRI_MAX_REQ: u8 = 0x08;
///
pub const PCI_PRI_ALLOC_REQ: u8 = 0x0c;
///
pub const PCI_TPH_CAPABILITIES: u16 = 4;
///
pub const PCI_LTR_MAX_SNOOP: u16 = 4;
///
pub const PCI_LTR_MAX_NOSNOOP: u16 = 6;
///
pub const PCI_SEC_LNKCTL3: u16 = 4;
///
pub const PCI_SEC_LNKCTL3_PERFORM_LINK_EQU: u8 = 0x01;
///
pub const PCI_SEC_LNKCTL3_LNK_EQU_REQ_INTR_EN: u8 = 0x02;
///
pub const PCI_SEC_LANE_ERR: u16 = 8;
///
pub const PCI_SEC_LANE_EQU_CTRL: u16 = 12;
///
pub const PCI_PASID_CAP: u8 = 0x04;
///
pub const PCI_PASID_CAP_EXEC: u8 = 0x02;
///
pub const PCI_PASID_CAP_PRIV: u8 = 0x04;
///
pub const PCI_PASID_CTRL: u8 = 0x06;
///
pub const PCI_PASID_CTRL_ENABLE: u8 = 0x01;
///
pub const PCI_PASID_CTRL_EXEC: u8 = 0x02;
///
pub const PCI_PASID_CTRL_PRIV: u8 = 0x04;
///
pub const PCI_DPC_CAP: u16 = 4;
///
pub const PCI_DPC_CAP_RP_EXT: u8 = 0x20;
///
pub const PCI_DPC_CAP_TLP_BLOCK: u8 = 0x40;
///
pub const PCI_DPC_CAP_SW_TRIGGER: u8 = 0x80;
///
pub const PCI_DPC_CAP_DL_ACT_ERR: u16 = 0x1000;
///
pub const PCI_DPC_CTL: u16 = 6;
///
pub const PCI_DPC_CTL_CMPL: u8 = 0x4;
///
pub const PCI_DPC_CTL_INT: u8 = 0x8;
///
pub const PCI_DPC_CTL_ERR_COR: u8 = 0x10;
///
pub const PCI_DPC_CTL_TLP: u8 = 0x20;
///
pub const PCI_DPC_CTL_SW_TRIGGER: u8 = 0x40;
///
pub const PCI_DPC_CTL_DL_ACTIVE: u8 = 0x80;
///
pub const PCI_DPC_STATUS: u16 = 8;
///
pub const PCI_DPC_STS_TRIGGER: u8 = 0x01;
///
pub const PCI_DPC_STS_INT: u8 = 0x08;
///
pub const PCI_DPC_STS_RP_BUSY: u8 = 0x10;
///
pub const PCI_DPC_SOURCE: u16 = 10;
///
pub const PCI_L1PM_SUBSTAT_CAP: u8 = 0x4;
///
pub const PCI_L1PM_SUBSTAT_CAP_PM_L12: u8 = 0x1;
///
pub const PCI_L1PM_SUBSTAT_CAP_PM_L11: u8 = 0x2;
///
pub const PCI_L1PM_SUBSTAT_CAP_ASPM_L12: u8 = 0x4;
///
pub const PCI_L1PM_SUBSTAT_CAP_ASPM_L11: u8 = 0x8;
///
pub const PCI_L1PM_SUBSTAT_CAP_L1PM_SUPP: u8 = 0x10;
///
pub const PCI_L1PM_SUBSTAT_CTL1: u8 = 0x8;
///
pub const PCI_L1PM_SUBSTAT_CTL1_PM_L12: u8 = 0x1;
///
pub const PCI_L1PM_SUBSTAT_CTL1_PM_L11: u8 = 0x2;
///
pub const PCI_L1PM_SUBSTAT_CTL1_ASPM_L12: u8 = 0x4;
///
pub const PCI_L1PM_SUBSTAT_CTL1_ASPM_L11: u8 = 0x8;
///
pub const PCI_L1PM_SUBSTAT_CTL2: u8 = 0xC;
///
pub const PCI_DOE_CAP: u8 = 0x4;
///
pub const PCI_DOE_CAP_INT_SUPP: u8 = 0x1;
///
pub const PCI_DOE_CTL: u8 = 0x8;
///
pub const PCI_DOE_CTL_ABORT: u8 = 0x1;
///
pub const PCI_DOE_CTL_INT: u8 = 0x2;
///
pub const PCI_DOE_STS: u8 = 0xC;
///
pub const PCI_DOE_STS_BUSY: u8 = 0x1;
///
pub const PCI_DOE_STS_INT: u8 = 0x2;
///
pub const PCI_DOE_STS_ERROR: u8 = 0x4;
///
pub const PCI_LMR_CAPS: u8 = 0x4;
///
pub const PCI_LMR_CAPS_DRVR: u8 = 0x1;
///
pub const PCI_LMR_PORT_STS: u8 = 0x6;
///
pub const PCI_LMR_PORT_STS_READY: u8 = 0x1;
///
pub const PCI_LMR_PORT_STS_SOFT_READY: u8 = 0x2;
///
pub const PCI_IDE_CAP: u8 = 0x4;
///
pub const PCI_IDE_CAP_LINK_IDE_SUPP: u8 = 0x1;
///
pub const PCI_IDE_CAP_SELECTIVE_IDE_SUPP: u8 = 0x2;
///
pub const PCI_IDE_CAP_FLOWTHROUGH_IDE_SUPP: u8 = 0x4;
///
pub const PCI_IDE_CAP_PARTIAL_HEADER_ENC_SUPP: u8 = 0x8;
///
pub const PCI_IDE_CAP_AGGREGATION_SUPP: u8 = 0x10;
///
pub const PCI_IDE_CAP_PCRC_SUPP: u8 = 0x20;
///
pub const PCI_IDE_CAP_IDE_KM_SUPP: u8 = 0x40;
///
pub const PCI_IDE_CAP_ALG_AES_GCM_256: u16 = 0;
///
pub const PCI_IDE_CTL: u8 = 0x8;
///
pub const PCI_IDE_CTL_FLOWTHROUGH_IDE: u8 = 0x4;
///
pub const PCI_IDE_LINK_STREAM: u8 = 0xC;
///
pub const PCI_IDE_LINK_CTL_EN: u8 = 0x1;
///
pub const PCI_IDE_LINK_CTL_PCRC_EN: u16 = 0x100;
///
pub const PCI_IDE_SEL_CTL_EN: u8 = 0x1;
///
pub const PCI_IDE_SEL_CTL_PCRC_EN: u16 = 0x100;
///
pub const PCI_IDE_SEL_RID_2_VALID: u8 = 0x1;
///
pub const PCI_IDE_SEL_ADDR_1_VALID: u8 = 0x1;
///
pub const PCI_CLASS_NOT_DEFINED: u16 = 0x0000;
///
pub const PCI_CLASS_NOT_DEFINED_VGA: u16 = 0x0001;
///
pub const PCI_BASE_CLASS_STORAGE: u8 = 0x01;
///
pub const PCI_CLASS_STORAGE_SCSI: u16 = 0x0100;
///
pub const PCI_CLASS_STORAGE_IDE: u16 = 0x0101;
///
pub const PCI_CLASS_STORAGE_FLOPPY: u16 = 0x0102;
///
pub const PCI_CLASS_STORAGE_IPI: u16 = 0x0103;
///
pub const PCI_CLASS_STORAGE_RAID: u16 = 0x0104;
///
pub const PCI_CLASS_STORAGE_ATA: u16 = 0x0105;
///
pub const PCI_CLASS_STORAGE_SATA: u16 = 0x0106;
///
pub const PCI_CLASS_STORAGE_SAS: u16 = 0x0107;
///
pub const PCI_CLASS_STORAGE_OTHER: u16 = 0x0180;
///
pub const PCI_BASE_CLASS_NETWORK: u8 = 0x02;
///
pub const PCI_CLASS_NETWORK_ETHERNET: u16 = 0x0200;
///
pub const PCI_CLASS_NETWORK_TOKEN_RING: u16 = 0x0201;
///
pub const PCI_CLASS_NETWORK_FDDI: u16 = 0x0202;
///
pub const PCI_CLASS_NETWORK_ATM: u16 = 0x0203;
///
pub const PCI_CLASS_NETWORK_ISDN: u16 = 0x0204;
///
pub const PCI_CLASS_NETWORK_OTHER: u16 = 0x0280;
///
pub const PCI_BASE_CLASS_DISPLAY: u8 = 0x03;
///
pub const PCI_CLASS_DISPLAY_VGA: u16 = 0x0300;
///
pub const PCI_CLASS_DISPLAY_XGA: u16 = 0x0301;
///
pub const PCI_CLASS_DISPLAY_3D: u16 = 0x0302;
///
pub const PCI_CLASS_DISPLAY_OTHER: u16 = 0x0380;
///
pub const PCI_BASE_CLASS_MULTIMEDIA: u8 = 0x04;
///
pub const PCI_CLASS_MULTIMEDIA_VIDEO: u16 = 0x0400;
///
pub const PCI_CLASS_MULTIMEDIA_AUDIO: u16 = 0x0401;
///
pub const PCI_CLASS_MULTIMEDIA_PHONE: u16 = 0x0402;
///
pub const PCI_CLASS_MULTIMEDIA_AUDIO_DEV: u16 = 0x0403;
///
pub const PCI_CLASS_MULTIMEDIA_OTHER: u16 = 0x0480;
///
pub const PCI_BASE_CLASS_MEMORY: u8 = 0x05;
///
pub const PCI_CLASS_MEMORY_RAM: u16 = 0x0500;
///
pub const PCI_CLASS_MEMORY_FLASH: u16 = 0x0501;
///
pub const PCI_CLASS_MEMORY_OTHER: u16 = 0x0580;
///
pub const PCI_BASE_CLASS_BRIDGE: u8 = 0x06;
///
pub const PCI_CLASS_BRIDGE_HOST: u16 = 0x0600;
///
pub const PCI_CLASS_BRIDGE_ISA: u16 = 0x0601;
///
pub const PCI_CLASS_BRIDGE_EISA: u16 = 0x0602;
///
pub const PCI_CLASS_BRIDGE_MC: u16 = 0x0603;
///
pub const PCI_CLASS_BRIDGE_PCI: u16 = 0x0604;
///
pub const PCI_CLASS_BRIDGE_PCMCIA: u16 = 0x0605;
///
pub const PCI_CLASS_BRIDGE_NUBUS: u16 = 0x0606;
///
pub const PCI_CLASS_BRIDGE_CARDBUS: u16 = 0x0607;
///
pub const PCI_CLASS_BRIDGE_RACEWAY: u16 = 0x0608;
///
pub const PCI_CLASS_BRIDGE_PCI_SEMI: u16 = 0x0609;
///
pub const PCI_CLASS_BRIDGE_IB_TO_PCI: u16 = 0x060a;
///
pub const PCI_CLASS_BRIDGE_OTHER: u16 = 0x0680;
///
pub const PCI_BASE_CLASS_COMMUNICATION: u8 = 0x07;
///
pub const PCI_CLASS_COMMUNICATION_SERIAL: u16 = 0x0700;
///
pub const PCI_CLASS_COMMUNICATION_PARALLEL: u16 = 0x0701;
///
pub const PCI_CLASS_COMMUNICATION_MSERIAL: u16 = 0x0702;
///
pub const PCI_CLASS_COMMUNICATION_MODEM: u16 = 0x0703;
///
pub const PCI_CLASS_COMMUNICATION_OTHER: u16 = 0x0780;
///
pub const PCI_BASE_CLASS_SYSTEM: u8 = 0x08;
///
pub const PCI_CLASS_SYSTEM_PIC: u16 = 0x0800;
///
pub const PCI_CLASS_SYSTEM_DMA: u16 = 0x0801;
///
pub const PCI_CLASS_SYSTEM_TIMER: u16 = 0x0802;
///
pub const PCI_CLASS_SYSTEM_RTC: u16 = 0x0803;
///
pub const PCI_CLASS_SYSTEM_PCI_HOTPLUG: u16 = 0x0804;
///
pub const PCI_CLASS_SYSTEM_OTHER: u16 = 0x0880;
///
pub const PCI_BASE_CLASS_INPUT: u8 = 0x09;
///
pub const PCI_CLASS_INPUT_KEYBOARD: u16 = 0x0900;
///
pub const PCI_CLASS_INPUT_PEN: u16 = 0x0901;
///
pub const PCI_CLASS_INPUT_MOUSE: u16 = 0x0902;
///
pub const PCI_CLASS_INPUT_SCANNER: u16 = 0x0903;
///
pub const PCI_CLASS_INPUT_GAMEPORT: u16 = 0x0904;
///
pub const PCI_CLASS_INPUT_OTHER: u16 = 0x0980;
///
pub const PCI_BASE_CLASS_DOCKING: u8 = 0x0a;
///
pub const PCI_CLASS_DOCKING_GENERIC: u16 = 0x0a00;
///
pub const PCI_CLASS_DOCKING_OTHER: u16 = 0x0a80;
///
pub const PCI_BASE_CLASS_PROCESSOR: u8 = 0x0b;
///
pub const PCI_CLASS_PROCESSOR_386: u16 = 0x0b00;
///
pub const PCI_CLASS_PROCESSOR_486: u16 = 0x0b01;
///
pub const PCI_CLASS_PROCESSOR_PENTIUM: u16 = 0x0b02;
///
pub const PCI_CLASS_PROCESSOR_ALPHA: u16 = 0x0b10;
///
pub const PCI_CLASS_PROCESSOR_POWERPC: u16 = 0x0b20;
///
pub const PCI_CLASS_PROCESSOR_MIPS: u16 = 0x0b30;
///
pub const PCI_CLASS_PROCESSOR_CO: u16 = 0x0b40;
///
pub const PCI_BASE_CLASS_SERIAL: u8 = 0x0c;
///
pub const PCI_CLASS_SERIAL_FIREWIRE: u16 = 0x0c00;
///
pub const PCI_CLASS_SERIAL_ACCESS: u16 = 0x0c01;
///
pub const PCI_CLASS_SERIAL_SSA: u16 = 0x0c02;
///
pub const PCI_CLASS_SERIAL_USB: u16 = 0x0c03;
///
pub const PCI_CLASS_SERIAL_FIBER: u16 = 0x0c04;
///
pub const PCI_CLASS_SERIAL_SMBUS: u16 = 0x0c05;
///
pub const PCI_CLASS_SERIAL_INFINIBAND: u16 = 0x0c06;
///
pub const PCI_BASE_CLASS_WIRELESS: u8 = 0x0d;
///
pub const PCI_CLASS_WIRELESS_IRDA: u16 = 0x0d00;
///
pub const PCI_CLASS_WIRELESS_CONSUMER_IR: u16 = 0x0d01;
///
pub const PCI_CLASS_WIRELESS_RF: u16 = 0x0d10;
///
pub const PCI_CLASS_WIRELESS_OTHER: u16 = 0x0d80;
///
pub const PCI_BASE_CLASS_INTELLIGENT: u8 = 0x0e;
///
pub const PCI_CLASS_INTELLIGENT_I2O: u16 = 0x0e00;
///
pub const PCI_BASE_CLASS_SATELLITE: u8 = 0x0f;
///
pub const PCI_CLASS_SATELLITE_TV: u16 = 0x0f00;
///
pub const PCI_CLASS_SATELLITE_AUDIO: u16 = 0x0f01;
///
pub const PCI_CLASS_SATELLITE_VOICE: u16 = 0x0f03;
///
pub const PCI_CLASS_SATELLITE_DATA: u16 = 0x0f04;
///
pub const PCI_BASE_CLASS_CRYPT: u8 = 0x10;
///
pub const PCI_CLASS_CRYPT_NETWORK: u16 = 0x1000;
///
pub const PCI_CLASS_CRYPT_ENTERTAINMENT: u16 = 0x1010;
///
pub const PCI_CLASS_CRYPT_OTHER: u16 = 0x1080;
///
pub const PCI_BASE_CLASS_SIGNAL: u8 = 0x11;
///
pub const PCI_CLASS_SIGNAL_DPIO: u16 = 0x1100;
///
pub const PCI_CLASS_SIGNAL_PERF_CTR: u16 = 0x1101;
///
pub const PCI_CLASS_SIGNAL_SYNCHRONIZER: u16 = 0x1110;
///
pub const PCI_CLASS_SIGNAL_OTHER: u16 = 0x1180;
///
pub const PCI_CLASS_OTHERS: u8 = 0xff;
///
pub const PCI_VENDOR_ID_INTEL: u16 = 0x8086;
///
pub const PCI_VENDOR_ID_COMPAQ: u16 = 0x0e11;
///
pub const PCI_ADDR_FLAG_MASK: u64 = 0xf;
///
pub const PCI_BASE_ADDRESS_IO_MASK: u32 = !(0x03 as u32);
///
pub const PCI_ADDR_MEM_MASK: u32 = !(0x0f as u32);
///
pub const PCI_IORESOURCE_PCI_EA_BEI: u32 = 1 << 5;
