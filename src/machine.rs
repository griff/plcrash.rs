
type cpu_type_t = u64;
type cpu_subtype_t = u64;

/*
 * Capability bits used in the definition of cpu_type.
 */
pub const CPU_ARCH_MASK  : cpu_type_t = 0xff000000; /* mask for architecture bits */
pub const CPU_ARCH_ABI64 : cpu_type_t = 0x01000000; /* 64 bit ABI */

/*
 *	Machine types known by all.
 */
pub const CPU_TYPE_VAX       : cpu_type_t = 1;
/* skip                      : cpu_type_t = 2; */
/* skip                      : cpu_type_t = 3; */
/* skip                      : cpu_type_t = 4; */
/* skip                      : cpu_type_t = 5; */
pub const CPU_TYPE_MC680x0   : cpu_type_t = 6;
pub const CPU_TYPE_X86       : cpu_type_t = 7;
pub const CPU_TYPE_I386      : cpu_type_t = CPU_TYPE_X86; /* compatibility */
pub const CPU_TYPE_X86_64    : cpu_type_t = (CPU_TYPE_X86 | CPU_ARCH_ABI64);

/* skip   CPU_TYPE_MIPS      : cpu_type_t =  8; */
/* skip                      : cpu_type_t = 9; */
pub const CPU_TYPE_MC98000   : cpu_type_t = 10;
pub const CPU_TYPE_HPPA      : cpu_type_t = 11;
pub const CPU_TYPE_ARM       : cpu_type_t = 12;
pub const CPU_TYPE_ARM64     : cpu_type_t = (CPU_TYPE_ARM | CPU_ARCH_ABI64);
pub const CPU_TYPE_MC88000   : cpu_type_t = 13;
pub const CPU_TYPE_SPARC     : cpu_type_t = 14;
pub const CPU_TYPE_I860      : cpu_type_t = 15;
/* skip   CPU_TYPE_ALPHA     : cpu_type_t = 16; */
/* skip                      : cpu_type_t = 17; */
pub const CPU_TYPE_POWERPC   : cpu_type_t = 18;
pub const CPU_TYPE_POWERPC64 : cpu_type_t = (CPU_TYPE_POWERPC | CPU_ARCH_ABI64);


/*
 *	Machine subtypes (these are defined here, instead of in a machine
 *	dependent directory, so that any program can get all definitions
 *	regardless of where is it compiled).
 */

/*
 * Capability bits used in the definition of cpu_subtype.
 */
pub const CPU_SUBTYPE_MASK  : cpu_subtype_t = 0xff000000; /* mask for feature flags */
pub const CPU_SUBTYPE_LIB64 : cpu_subtype_t = 0x80000000; /* 64 bit libraries */


/*
 *	Object files that are hand-crafted to run on any
 *	implementation of an architecture are tagged with
 *	CPU_SUBTYPE_MULTIPLE.  This functions essentially the same as
 *	the "ALL" subtype of an architecture except that it allows us
 *	to easily find object files that may need to be modified
 *	whenever a new implementation of an architecture comes out.
 *
 *	It is the responsibility of the implementor to make sure the
 *	software handles unsupported implementations elegantly.
 */
pub const CPU_SUBTYPE_LITTLE_ENDIAN : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_BIG_ENDIAN    : cpu_subtype_t = 1;

/*
 *	VAX subtypes (these do *not* necessary conform to the actual cpu
 *	ID assigned by DEC available via the SID register).
 */

pub const CPU_SUBTYPE_VAX_ALL : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_VAX780  : cpu_subtype_t = 1;
pub const CPU_SUBTYPE_VAX785  : cpu_subtype_t = 2;
pub const CPU_SUBTYPE_VAX750  : cpu_subtype_t = 3;
pub const CPU_SUBTYPE_VAX730  : cpu_subtype_t = 4;
pub const CPU_SUBTYPE_UVAXI   : cpu_subtype_t = 5;
pub const CPU_SUBTYPE_UVAXII  : cpu_subtype_t = 6;
pub const CPU_SUBTYPE_VAX8200 : cpu_subtype_t = 7;
pub const CPU_SUBTYPE_VAX8500 : cpu_subtype_t = 8;
pub const CPU_SUBTYPE_VAX8600 : cpu_subtype_t = 9;
pub const CPU_SUBTYPE_VAX8650 : cpu_subtype_t = 10;
pub const CPU_SUBTYPE_VAX8800 : cpu_subtype_t = 11;
pub const CPU_SUBTYPE_UVAXIII : cpu_subtype_t = 12;

/*
 * 	680x0 subtypes
 *
 * The subtype definitions here are unusual for historical reasons.
 * NeXT used to consider 68030 code as generic 68000 code.  For
 * backwards compatability:
 *
 *	CPU_SUBTYPE_MC68030 symbol has been preserved for source code
 *	compatability.
 *
 *	CPU_SUBTYPE_MC680x0_ALL has been defined to be the same
 *	subtype as CPU_SUBTYPE_MC68030 for binary comatability.
 *
 *	CPU_SUBTYPE_MC68030_ONLY has been added to allow new object
 *	files to be tagged as containing 68030-specific instructions.
 */

pub const CPU_SUBTYPE_MC680x0_ALL  : cpu_subtype_t = 1;
pub const CPU_SUBTYPE_MC68030      : cpu_subtype_t = 1; /* compat */
pub const CPU_SUBTYPE_MC68040      : cpu_subtype_t = 2;
pub const CPU_SUBTYPE_MC68030_ONLY : cpu_subtype_t = 3;

/*
 *	I386 subtypes
 */
macro_rules! cpu_subtype_intel {
    ($f:expr, $m:expr) => (($f) + (($m) << 4))
}

pub const CPU_SUBTYPE_I386_ALL       : cpu_subtype_t = cpu_subtype_intel!(3, 0);
pub const CPU_SUBTYPE_386            : cpu_subtype_t = cpu_subtype_intel!(3, 0);
pub const CPU_SUBTYPE_486            : cpu_subtype_t = cpu_subtype_intel!(4, 0);
pub const CPU_SUBTYPE_486SX          : cpu_subtype_t = cpu_subtype_intel!(4, 8); // 8 << 4 = 128
pub const CPU_SUBTYPE_586            : cpu_subtype_t = cpu_subtype_intel!(5, 0);
pub const CPU_SUBTYPE_PENT           : cpu_subtype_t = cpu_subtype_intel!(5, 0);
pub const CPU_SUBTYPE_PENTPRO        : cpu_subtype_t = cpu_subtype_intel!(6, 1);
pub const CPU_SUBTYPE_PENTII_M3      : cpu_subtype_t = cpu_subtype_intel!(6, 3);
pub const CPU_SUBTYPE_PENTII_M5      : cpu_subtype_t = cpu_subtype_intel!(6, 5);
pub const CPU_SUBTYPE_CELERON        : cpu_subtype_t = cpu_subtype_intel!(7, 6);
pub const CPU_SUBTYPE_CELERON_MOBILE : cpu_subtype_t = cpu_subtype_intel!(7, 7);
pub const CPU_SUBTYPE_PENTIUM_3      : cpu_subtype_t = cpu_subtype_intel!(8, 0);
pub const CPU_SUBTYPE_PENTIUM_3_M    : cpu_subtype_t = cpu_subtype_intel!(8, 1);
pub const CPU_SUBTYPE_PENTIUM_3_XEON : cpu_subtype_t = cpu_subtype_intel!(8, 2);
pub const CPU_SUBTYPE_PENTIUM_M      : cpu_subtype_t = cpu_subtype_intel!(9, 0);
pub const CPU_SUBTYPE_PENTIUM_4      : cpu_subtype_t = cpu_subtype_intel!(10, 0);
pub const CPU_SUBTYPE_PENTIUM_4_M    : cpu_subtype_t = cpu_subtype_intel!(10, 1);
pub const CPU_SUBTYPE_ITANIUM        : cpu_subtype_t = cpu_subtype_intel!(11, 0);
pub const CPU_SUBTYPE_ITANIUM_2      : cpu_subtype_t = cpu_subtype_intel!(11, 1);
pub const CPU_SUBTYPE_XEON           : cpu_subtype_t = cpu_subtype_intel!(12, 0);
pub const CPU_SUBTYPE_XEON_MP        : cpu_subtype_t = cpu_subtype_intel!(12, 1);

macro_rules! cpu_subtype_intel_family {
    ($x:expr) => (($x) & 15)
}
pub const CPU_SUBTYPE_INTEL_FAMILY_MAX : cpu_subtype_t = 15;

macro_rules! cpu_subtype_intel_model {
    ($x:expr) => ((x) >> 4)
}
pub const CPU_SUBTYPE_INTEL_MODEL_ALL : cpu_subtype_t = 0;

/*
 *	X86 subtypes.
 */

pub const CPU_SUBTYPE_X86_ALL    : cpu_subtype_t = 3;
pub const CPU_SUBTYPE_X86_64_ALL : cpu_subtype_t = 3;
pub const CPU_SUBTYPE_X86_ARCH1  : cpu_subtype_t = 4;
pub const CPU_SUBTYPE_X86_64_H   : cpu_subtype_t = 8; /* Haswell feature subset */


/*
 *	Mips subtypes.
 */

pub const CPU_SUBTYPE_MIPS_ALL    : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_MIPS_R2300  : cpu_subtype_t = 1;
pub const CPU_SUBTYPE_MIPS_R2600  : cpu_subtype_t = 2;
pub const CPU_SUBTYPE_MIPS_R2800  : cpu_subtype_t = 3;
pub const CPU_SUBTYPE_MIPS_R2000a : cpu_subtype_t = 4; /* pmax */
pub const CPU_SUBTYPE_MIPS_R2000  : cpu_subtype_t = 5;
pub const CPU_SUBTYPE_MIPS_R3000a : cpu_subtype_t = 6; /* 3max */
pub const CPU_SUBTYPE_MIPS_R3000  : cpu_subtype_t = 7;

/*
 *	MC98000 (PowerPC) subtypes
 */
pub const CPU_SUBTYPE_MC98000_ALL : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_MC98601     : cpu_subtype_t = 1;

/*
 *	HPPA subtypes for Hewlett-Packard HP-PA family of
 *	risc processors. Port by NeXT to 700 series.
 */

pub const CPU_SUBTYPE_HPPA_ALL    : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_HPPA_7100   : cpu_subtype_t = 0; /* compat */
pub const CPU_SUBTYPE_HPPA_7100LC : cpu_subtype_t = 1;

/*
 *	MC88000 subtypes.
 */
pub const CPU_SUBTYPE_MC88000_ALL : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_MC88100     : cpu_subtype_t = 1;
pub const CPU_SUBTYPE_MC88110     : cpu_subtype_t = 2;

/*
 *	SPARC subtypes
 */
pub const CPU_SUBTYPE_SPARC_ALL : cpu_subtype_t = 0;

/*
 *	I860 subtypes
 */
pub const CPU_SUBTYPE_I860_ALL : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_I860_860 : cpu_subtype_t = 1;

/*
 *	PowerPC subtypes
 */
pub const CPU_SUBTYPE_POWERPC_ALL   : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_POWERPC_601   : cpu_subtype_t = 1;
pub const CPU_SUBTYPE_POWERPC_602   : cpu_subtype_t = 2;
pub const CPU_SUBTYPE_POWERPC_603   : cpu_subtype_t = 3;
pub const CPU_SUBTYPE_POWERPC_603e  : cpu_subtype_t = 4;
pub const CPU_SUBTYPE_POWERPC_603ev : cpu_subtype_t = 5;
pub const CPU_SUBTYPE_POWERPC_604   : cpu_subtype_t = 6;
pub const CPU_SUBTYPE_POWERPC_604e  : cpu_subtype_t = 7;
pub const CPU_SUBTYPE_POWERPC_620   : cpu_subtype_t = 8;
pub const CPU_SUBTYPE_POWERPC_750   : cpu_subtype_t = 9;
pub const CPU_SUBTYPE_POWERPC_7400  : cpu_subtype_t = 10;
pub const CPU_SUBTYPE_POWERPC_7450  : cpu_subtype_t = 11;
pub const CPU_SUBTYPE_POWERPC_970   : cpu_subtype_t = 100;

/*
 *	ARM subtypes
 */
pub const CPU_SUBTYPE_ARM_ALL    : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_ARM_V4T    : cpu_subtype_t = 5;
pub const CPU_SUBTYPE_ARM_V6     : cpu_subtype_t = 6;
pub const CPU_SUBTYPE_ARM_V5TEJ  : cpu_subtype_t = 7;
pub const CPU_SUBTYPE_ARM_XSCALE : cpu_subtype_t = 8;
pub const CPU_SUBTYPE_ARM_V7     : cpu_subtype_t = 9;
pub const CPU_SUBTYPE_ARM_V7F    : cpu_subtype_t = 10; /* Cortex A9 */
pub const CPU_SUBTYPE_ARM_V7S    : cpu_subtype_t = 11; /* Swift */
pub const CPU_SUBTYPE_ARM_V7K    : cpu_subtype_t = 12;
pub const CPU_SUBTYPE_ARM_V6M    : cpu_subtype_t = 14; /* Not meant to be run under xnu */
pub const CPU_SUBTYPE_ARM_V7M    : cpu_subtype_t = 15; /* Not meant to be run under xnu */
pub const CPU_SUBTYPE_ARM_V7EM   : cpu_subtype_t = 16; /* Not meant to be run under xnu */

pub const CPU_SUBTYPE_ARM_V8     : cpu_subtype_t = 13;

/*
 *  ARM64 subtypes
 */
pub const CPU_SUBTYPE_ARM64_ALL : cpu_subtype_t = 0;
pub const CPU_SUBTYPE_ARM64_V8  : cpu_subtype_t = 1;
