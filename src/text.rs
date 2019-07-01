use std::cmp;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::Path;

use chrono::naive::NaiveDateTime;
use failure::ResultExt;
use uuid::Uuid;

use super::protos::crash_report::*;
use super::machine::*;
use super::symbolicate::Symbolicate;
use crate::error::{Error, ErrorKind};

pub fn text_report(report: &CrashReport, symbolicate: Option<&Symbolicate>) -> Result<String, Error> {
    let mut text = String::new();
    let mut lp64 = true;

    /* Header */

    /* Map to apple style OS nane */
    let os_name = match report.get_system_info().get_operating_system() {
        CrashReport_SystemInfo_OperatingSystem::MAC_OS_X => "Mac OS X",
        CrashReport_SystemInfo_OperatingSystem::IPHONE_OS => "iPhone OS",
        CrashReport_SystemInfo_OperatingSystem::IPHONE_SIMULATOR => "Mac OS X",
        CrashReport_SystemInfo_OperatingSystem::OS_UNKNOWN => "Unknown",
    };

    /* Map to Apple-style code type, and mark whether architecture is LP64 (64-bit) */
    let mut code_type = "";
    {
        /* Attempt to derive the code type from the binary images */
        for image in report.get_binary_images() {
            /* Skip images with no specified type */
            if !image.has_code_type() {
                continue;
            }

            /* Skip unknown encodings */
            if image.get_code_type().get_encoding() != CrashReport_Processor_TypeEncoding::TYPE_ENCODING_MACH {
                continue;
            }

            match image.get_code_type().get_field_type() {
                CPU_TYPE_ARM => {
                    code_type = "ARM";
                    lp64 = false;
                },
                CPU_TYPE_ARM64 => {
                    code_type = "ARM-64";
                    lp64 = true;
                },
                CPU_TYPE_X86 => {
                    code_type = "X86";
                    lp64 = false;
                },
                CPU_TYPE_X86_64 => {
                    code_type = "X86-64";
                    lp64 = true;
                },
                CPU_TYPE_POWERPC => {
                    code_type = "PPC";
                    lp64 = false;
                },
                // Do nothing, handled below.
                _ => {}
            }

            /* Stop immediately if code type was discovered */
            if code_type != "" {
                break;
            }
        }

        /* If we were unable to determine the code type, fall back on the legacy architecture value. */
        if code_type == "" {
            match report.get_system_info().get_architecture() {
                Architecture::ARMV6 | Architecture::ARMV7 => {
                    code_type = "ARM";
                    lp64 = false;
                },
                Architecture::X86_32 => {
                    code_type = "X86";
                    lp64 = false;
                },
                Architecture::X86_64 => {
                    code_type = "X86-64";
                    lp64 = true;
                },
                Architecture::PPC => {
                    code_type = "PPC";
                    lp64 = false;
                },
                _ => {
                    code_type = "Unknown";
                    lp64 = true;
                },
            }
        }

    }

    {
        let hardware_model = if report.has_machine_info() && report.get_machine_info().has_model() {
            String::from(report.get_machine_info().get_model())
        } else {
            String::from("???")
        };

        let incident_identifier = if report.get_report_info().has_uuid() {
            if let Ok(uuid) = Uuid::from_slice(report.get_report_info().get_uuid()) {
                uuid
            } else {
                Uuid::new_v4()
            }
        } else {
            Uuid::new_v4()
        };

        writeln!(text, "Incident Identifier: {}", incident_identifier).unwrap();
        writeln!(text, "CrashReporter Key:   TODO").unwrap();
        writeln!(text, "Hardware Model:      {}", hardware_model).unwrap();
    }

    /* Application and process info */
    {
        let mut process_name = String::from("???");
        let mut process_id = String::from("???");
        let mut process_path = String::from("???");
        let mut parent_process_name = String::from("???");
        let mut parent_process_id = String::from("???");

        /* Process information was not available in earlier crash report versions */
        if report.has_process_info() {
            let process_info = report.get_process_info();

            /* Process Name */
            if process_info.has_process_name() {
                process_name = process_info.get_process_name().into();
            }

            /* PID */
            if process_info.has_process_id() {
                process_id = format!("{}", process_info.get_process_id());
            }

            /* Process Path */
            if process_info.has_process_path() {
                process_path = process_info.get_process_path().into();
            }

            /* Parent Process Name */
            if process_info.has_parent_process_name() {
                parent_process_name = process_info.get_parent_process_name().into();
            }

            /* Parent Process ID */
            if process_info.has_parent_process_id() {
                parent_process_id = format!("{}", process_info.get_parent_process_id());
            }
        }

        let application_info = report.get_application_info();
        writeln!(text, "Process:         {} [{}]", process_name, process_id).unwrap();
        writeln!(text, "Path:            {}", process_path).unwrap();
        writeln!(text, "Identifier:      {}", application_info.get_identifier()).unwrap();
        writeln!(text, "Version:         {}", application_info.get_version()).unwrap();
        writeln!(text, "Code Type:       {}", code_type).unwrap();
        writeln!(text, "Parent Process:  {} [{}]", parent_process_name, parent_process_id).unwrap();
    }

    writeln!(text).unwrap();

    /* System info */
    {
        let system_info = report.get_system_info();
        let os_build = if system_info.has_os_build() {
            system_info.get_os_build().into()
        } else {
            String::from("???")
        };

        let timestamp = system_info.get_timestamp();
        if timestamp > 0 {
            writeln!(text, "Date/Time:       {}", NaiveDateTime::from_timestamp(timestamp, 0)).unwrap();
        } else {
            writeln!(text, "Date/Time:       ???").unwrap();
        }
        writeln!(text, "OS Version:      {} {} ({})", os_name, system_info.get_os_version(), os_build).unwrap();
        writeln!(text, "Report Version:  104").unwrap();
    }

    writeln!(text).unwrap();

    /* Exception code */
    let signal_info = report.get_signal();
    writeln!(text, "Exception Type:  {}", signal_info.get_name()).unwrap();
    writeln!(text, "Exception Codes: {} at {:#x}", signal_info.get_code(), signal_info.get_address()).unwrap();

    for thread in report.get_threads() {
        if thread.get_crashed() {
            writeln!(text, "Crashed Thread: {}", thread.get_thread_number()).unwrap();
            break;
        }
    }

    writeln!(text).unwrap();

    /* Uncaught Exception */
    if report.has_exception() {
        writeln!(text, "Application Specific Information:").unwrap();
        writeln!(text, "*** Terminating app due to uncaught exception '{}', reason: '{}'",
            report.get_exception().get_name(), report.get_exception().get_reason()
        ).unwrap();

        writeln!(text).unwrap();
    }

    /* If an exception stack trace is available, output an Apple-compatible backtrace. */
    if report.has_exception() && report.get_exception().get_frames().len() > 0 {
        let frames = report.get_exception().get_frames();

        /* Create the header. */
        writeln!(text, "Last Exception Backtrace:").unwrap();

        /* Write out the frames. In raw reports, Apple writes this out as a simple list of PCs. In the minimally
         * post-processed report, Apple writes this out as full frame entries. We use the latter format. */
        for idx in 0..frames.len() {
            let frame = &frames[idx];
            writeln!(text, "{}", format_stack_frame(&frame, idx, report, lp64, &symbolicate)?).unwrap();
        }
        writeln!(text).unwrap();
    }

    /* Threads */
    let mut the_crashed_thread = None;
    let mut max_thread_num = 0;
    for thread in report.get_threads() {
        if thread.get_crashed() {
            writeln!(text, "Thread {} Crashed:", thread.get_thread_number()).unwrap();
            the_crashed_thread = Some(thread.clone());
        } else {
            writeln!(text, "Thread {}:", thread.get_thread_number()).unwrap();
        }
        let frames = thread.get_frames();
        for idx in 0..frames.len() {
            let frame = &frames[idx];
            writeln!(text, "{}", format_stack_frame(&frame, idx, report, lp64, &symbolicate)?).unwrap();
        }
        writeln!(text).unwrap();

        /* Track the highest thread number */
        max_thread_num = cmp::max(max_thread_num, thread.get_thread_number());
    }

    /* Registers */
    if let Some(crashed_thread) = the_crashed_thread {
        writeln!(text, "Thread {} crashed with {} Thread State:",
            crashed_thread.get_thread_number(), code_type
        ).unwrap();

        let mut reg_column = 0;
        for reg in crashed_thread.get_registers() {
            /* Remap register names to match Apple's crash reports */
            let mut reg_name = reg.get_name();
            if report.has_machine_info() && report.get_machine_info().get_processor().get_encoding() == CrashReport_Processor_TypeEncoding::TYPE_ENCODING_MACH {
                let pinfo = report.get_machine_info().get_processor();
                let arch_type = pinfo.get_field_type() & !CPU_ARCH_MASK;

                /* Apple uses 'ip' rather than 'r12' on ARM */
                if arch_type == CPU_TYPE_ARM && reg_name == "r12" {
                    reg_name = "ip";
                }
            }
            /* Use 32-bit or 64-bit fixed width format for the register values */
            if lp64 {
                write!(text, "{:6}: {:#018x} ", reg_name, reg.get_value()).unwrap();
            } else {
                write!(text, "{:6}: {:#010x} ", reg_name, reg.get_value()).unwrap();
            }

            reg_column += 1;
            if reg_column == 4 {
                writeln!(text).unwrap();
                reg_column = 0;
            }
        }

        if reg_column != 0 {
            writeln!(text).unwrap();
        }

        writeln!(text).unwrap();
    }

    /* Images. The iPhone crash report format sorts these in ascending order, by the base address */
    writeln!(text, "Binary Images:").unwrap();
    let mut images = report.get_binary_images().to_vec();
    images.sort_by_key(|a| a.get_base_address() );
    for image in images {
        /* Fetch the UUID if it exists */
        let uuid = if image.has_uuid() {
            if let Ok(uuid) = Uuid::from_slice(image.get_uuid()) {
                format!("{}", uuid)
            } else {
                String::from("???")
            }
        } else {
            String::from("???")
        };

        /* Determine the architecture string */
        let arch_name = if image.has_code_type() && image.get_code_type().get_encoding() == CrashReport_Processor_TypeEncoding::TYPE_ENCODING_MACH {
            match image.get_code_type().get_field_type() {
                CPU_TYPE_ARM => {
                    match image.get_code_type().get_subtype() {
                        CPU_SUBTYPE_ARM_V6 => "armv6",
                        CPU_SUBTYPE_ARM_V7 => "armv7",
                        CPU_SUBTYPE_ARM_V7S => "armv7s",
                        _ => "arm-unknown",
                    }
                },
                CPU_TYPE_ARM64 => {
                    match image.get_code_type().get_subtype() {
                        CPU_SUBTYPE_ARM_ALL => "arm64",
                        CPU_SUBTYPE_ARM_V8 => "armv8",
                        _ => "arm64-unknown",
                    }
                },
                CPU_TYPE_X86 => "i386",
                CPU_TYPE_X86_64 => "x86_64",
                CPU_TYPE_POWERPC => "powerpc",
                _ => "???"
            }
        } else {
            "???"
        };

        /* Determine if this is the main executable */
        let binary_designator = if image.get_name() == report.get_process_info().get_process_path() {
            "+"
        } else {
            " "
        };

        /* base_address - terminating_address [designator]file_name arch <uuid> file_path */
        if lp64 {
            writeln!(text, "{:#18x} - {:#18x} {}{} {}  <{}> {}",
                image.get_base_address(),
                image.get_base_address() + cmp::max(1, image.get_size()) - 1, // The Apple format uses an inclusive range
                binary_designator,
                Path::new(Path::new(image.get_name()).file_name().unwrap_or(OsStr::new("???"))).display(),
                arch_name,
                uuid,
                image.get_name()
            ).unwrap();
        } else {
            writeln!(text, "{:#10x} - {:#10x} {}{} {}  <{}> {}",
                image.get_base_address(),
                image.get_base_address() + cmp::max(1, image.get_size()) - 1, // The Apple format uses an inclusive range
                binary_designator,
                Path::new(Path::new(image.get_name()).file_name().unwrap_or(OsStr::new("???"))).display(),
                arch_name,
                uuid,
                image.get_name()
            ).unwrap();
        }
    }

    Ok(text)
}

fn format_stack_frame(frame: &CrashReport_Thread_StackFrame, idx: usize, report: &CrashReport, lp64: bool, symbolicate: &Option<&Symbolicate>) -> Result<String, Error> {
    /* Base image address containing instrumention pointer, offset of the IP from that base
     * address, and the associated image name */
    let mut base_address = 0x0;
    let mut pc_offset = 0x0;
    let mut image_name = Path::new("???").display();
    let mut dsym_frame = None;

    if let Some(image) = image_for_address(report, frame.get_pc()) {
        /* Fetch the dSYM if it exists */
        let lookup = if image.has_uuid() {
            if let Ok(uuid) = Uuid::from_slice(image.get_uuid()) {
                symbolicate.and_then(|s| s.get(&uuid))
            } else {
                None
            }
        } else {
            None
        };

        image_name = Path::new(Path::new(image.get_name()).file_name().unwrap_or(OsStr::new("???"))).display();
        base_address = image.get_base_address();
        pc_offset = frame.get_pc() - base_address;
        if let Some(lookup) = lookup {
            dsym_frame = lookup
                .find_frames(pc_offset)?
                .nth(0)
                .transpose()
                .context(ErrorKind::Probe(lookup.name.clone(), pc_offset))?;
        }
    }

    /* If symbol info is available, the format used in Apple's reports is Sym + OffsetFromSym. Otherwise,
     * the format used is imageBaseAddress + offsetToIP */
    let symbol_string = if frame.has_symbol() {
        let mut symbol_name = frame.get_symbol().get_name();

        /* Apple strips the _ symbol prefix in their reports. Only OS X makes use of an
         * underscore symbol prefix by default. */
        if symbol_name.starts_with("_") && symbol_name.len() > 1 {
            match report.get_system_info().get_operating_system() {
                CrashReport_SystemInfo_OperatingSystem::MAC_OS_X |
                CrashReport_SystemInfo_OperatingSystem::IPHONE_OS |
                CrashReport_SystemInfo_OperatingSystem::IPHONE_SIMULATOR => {
                    let (_, s) = symbol_name.split_at(1);
                    symbol_name = s;
                },
                _ => {
                    /* Symbol prefix rules are unknown for this OS! */
                }
            }
        }

        let sym_offset = frame.get_pc() - frame.get_symbol().get_start_address();
        let location = if let Some(loc) = dsym_frame.and_then(|f| f.location() ) {
            format!(" ({})", loc)
        } else {
            String::from("")
        };
        format!("{} + {}{}", symbol_name, sym_offset, location)
    } else {
        let location = if let Some(df) = dsym_frame {
            format!(" {}", df)
        } else {
            String::from("")
        };
        format!("{:#x} + {}{}", base_address, pc_offset, location)
    };

    /* Note that width specifiers are ignored for %@, but work for C strings.
     * UTF-8 is not correctly handled with %s (it depends on the system encoding), but
     * UTF-16 is supported via %S, so we use it here */
    if lp64 {
        Ok(format!("{:<4}{:<35} {:#018x} {}",
            idx,
            image_name,
            frame.get_pc(),
            symbol_string))
    } else {
        Ok(format!("{:<4}{:<35} {:#10x} {}",
            idx,
            image_name,
            frame.get_pc(),
            symbol_string))
    }
}

fn image_for_address(report: &CrashReport, address: u64) -> Option<&CrashReport_BinaryImage> {
    for image in report.get_binary_images() {
        let base_address = image.get_base_address();
        if base_address <= address && address < (base_address + image.get_size()) {
            return Some(image);
        }
    }

    /* Not found */
    None
}


mod tests {

}
