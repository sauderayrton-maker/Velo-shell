//! CPU and memory usage, sampled from `/proc`.

use std::cell::Cell;
use std::fs;

use gtk4::prelude::*;

#[derive(Clone, Copy, Default)]
struct CpuTotals {
    idle: u64,
    total: u64,
}

pub fn build() -> gtk4::Box {
    let root = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(14).css_classes(vec!["module", "stats-module"]).build();

    let cpu_value = gtk4::Label::builder().css_classes(vec!["stat-value"]).build();
    let mem_value = gtk4::Label::builder().css_classes(vec!["stat-value"]).build();

    root.append(&stat_widget("CPU", &cpu_value));
    root.append(&stat_widget("RAM", &mem_value));

    let prev_cpu = Cell::new(read_cpu_totals());
    update(&cpu_value, &mem_value, &prev_cpu);

    glib::timeout_add_seconds_local(2, move || {
        update(&cpu_value, &mem_value, &prev_cpu);
        glib::ControlFlow::Continue
    });

    root
}

fn stat_widget(name: &str, value: &gtk4::Label) -> gtk4::Box {
    let name_label = gtk4::Label::builder().label(name).css_classes(vec!["stat-name"]).build();
    let row = gtk4::Box::builder().orientation(gtk4::Orientation::Horizontal).spacing(4).build();
    row.append(&name_label);
    row.append(value);
    row
}

fn update(cpu_value: &gtk4::Label, mem_value: &gtk4::Label, prev_cpu: &Cell<CpuTotals>) {
    let prev = prev_cpu.get();
    let cur = read_cpu_totals();

    let total_delta = cur.total.saturating_sub(prev.total);
    let idle_delta = cur.idle.saturating_sub(prev.idle);
    let usage = if total_delta > 0 { 100.0 * (1.0 - idle_delta as f64 / total_delta as f64) } else { 0.0 };
    prev_cpu.set(cur);
    cpu_value.set_label(&format!("{:.0}%", usage.clamp(0.0, 100.0)));

    if let Some(mem) = read_mem_percent() {
        mem_value.set_label(&format!("{mem:.0}%"));
    }
}

/// Reads the aggregate `cpu` line of `/proc/stat`: `(idle + iowait, total)`.
fn read_cpu_totals() -> CpuTotals {
    let content = fs::read_to_string("/proc/stat").unwrap_or_default();
    let fields: Vec<u64> = content.lines().next().unwrap_or("").split_whitespace().skip(1).filter_map(|f| f.parse().ok()).collect();

    let idle = fields.get(3).copied().unwrap_or(0) + fields.get(4).copied().unwrap_or(0);
    let total = fields.iter().sum();
    CpuTotals { idle, total }
}

/// Percentage of memory in use, from `MemTotal` and `MemAvailable` in `/proc/meminfo`.
fn read_mem_percent() -> Option<f64> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;

    let mut total = None;
    let mut available = None;
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v.trim().split_whitespace().next().and_then(|n| n.parse::<f64>().ok());
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            available = v.trim().split_whitespace().next().and_then(|n| n.parse::<f64>().ok());
        }
    }

    let (total, available) = (total?, available?);
    (total > 0.0).then(|| 100.0 * (1.0 - available / total))
}
