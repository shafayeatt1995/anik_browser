use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWarpMouseCursorPosition(new_cursor_position: CGPoint) -> i32;
    fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
    fn CGEventCreateScrollWheelEvent2(
        source: *const std::ffi::c_void,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> *mut std::ffi::c_void;
    fn CGEventPost(tap: u32, event: *const std::ffi::c_void);
    fn CFRelease(cf: *const std::ffi::c_void);
}

// Get actual current mouse location on screen
pub fn get_current_mouse_position() -> CGPoint {
    unsafe {
        let ev = CGEventCreate(std::ptr::null());
        if !ev.is_null() {
            let loc = CGEventGetLocation(ev);
            CFRelease(ev);
            return loc;
        }
    }
    CGPoint::new(400.0, 400.0)
}

// Move mouse like a real human: Bézier curvature, speed variations, and slight overshoot
pub fn human_like_move_mouse(
    start: CGPoint,
    end: CGPoint,
    auto_active: &Arc<AtomicBool>,
) {
    let mut rng = rand::thread_rng();
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 2.0 {
        return;
    }

    // Determine movement style randomly:
    // 0: Fast flick, 1: Smooth leisurely glide, 2: Wandering/hesitant curve
    let style = rng.gen_range(0..3);

    // Number of intermediate steps based on distance & style
    let steps = match style {
        0 => (dist / 14.0).clamp(18.0, 45.0) as usize, // fast move
        1 => (dist / 7.0).clamp(35.0, 90.0) as usize,  // smooth, steady move
        _ => (dist / 5.0).clamp(50.0, 130.0) as usize, // relaxed human exploration
    };

    // Calculate perpendicular offset for realistic natural hand curvature
    let perp_x = -dy / dist;
    let perp_y = dx / dist;

    // Random curve intensity (sometimes almost straight, sometimes prominently curved)
    let curve_mag = if rng.gen_bool(0.25) {
        // Nearly straight line with subtle micro-deviations
        rng.gen_range(-15.0..15.0)
    } else {
        // Curvy arc
        rng.gen_range(-1.0..1.0) * dist.min(180.0) * 0.45
    };

    // Control point 1 (around 25% - 40% of the path)
    let cp1_t = rng.gen_range(0.25..0.45);
    let cp1_x = start.x + dx * cp1_t + perp_x * curve_mag + rng.gen_range(-15.0..15.0);
    let cp1_y = start.y + dy * cp1_t + perp_y * curve_mag + rng.gen_range(-15.0..15.0);

    // Control point 2 (around 60% - 80% of the path, with natural counter-balance or continuation)
    let cp2_t = rng.gen_range(0.60..0.85);
    let counter_curv = if rng.gen_bool(0.4) { -curve_mag * 0.5 } else { curve_mag * 0.7 };
    let cp2_x = start.x + dx * cp2_t + perp_x * counter_curv + rng.gen_range(-15.0..15.0);
    let cp2_y = start.y + dy * cp2_t + perp_y * counter_curv + rng.gen_range(-15.0..15.0);

    let base_sleep_ms = match style {
        0 => rng.gen_range(5..10),  // faster updates
        1 => rng.gen_range(10..18), // standard smooth updates
        _ => rng.gen_range(14..24), // slow, deliberate movement
    };

    if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
        for i in 1..=steps {
            if !auto_active.load(Ordering::SeqCst) {
                break;
            }

            let t = i as f64 / steps as f64;

            // Human acceleration curve: slow start, quick sweep, decelerate at destination
            let eased_t = if style == 0 {
                // Quick start then deceleration
                t * (2.0 - t)
            } else {
                // Smooth bell-shaped acceleration (SmoothStep / Sigmoidal)
                t * t * (3.0 - 2.0 * t)
            };

            // Cubic Bézier calculation: B(t) = (1-t)^3*P0 + 3(1-t)^2*t*P1 + 3(1-t)*t^2*P2 + t^3*P3
            let u = 1.0 - eased_t;
            let tt = eased_t * eased_t;
            let uu = u * u;
            let uuu = uu * u;
            let ttt = tt * eased_t;

            let cur_x = uuu * start.x
                + 3.0 * uu * eased_t * cp1_x
                + 3.0 * u * tt * cp2_x
                + ttt * end.x;

            let cur_y = uuu * start.y
                + 3.0 * uu * eased_t * cp1_y
                + 3.0 * u * tt * cp2_y
                + ttt * end.y;

            // Add subtle micro-jitter like a real human hand
            let jitter_x = if i < steps - 3 { rng.gen_range(-0.7..0.7) } else { 0.0 };
            let jitter_y = if i < steps - 3 { rng.gen_range(-0.7..0.7) } else { 0.0 };

            let pt = CGPoint::new(cur_x + jitter_x, cur_y + jitter_y);

            unsafe {
                let _ = CGWarpMouseCursorPosition(pt);
            }

            if let Ok(move_ev) = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::MouseMoved,
                pt,
                CGMouseButton::Left,
            ) {
                move_ev.post(CGEventTapLocation::HID);
            }

            // Variable sleep for non-robotic timing
            let jitter_sleep = rng.gen_range(0..4);
            thread::sleep(Duration::from_millis((base_sleep_ms + jitter_sleep).max(2)));
        }

        // Ensure final exact target position
        unsafe {
            let _ = CGWarpMouseCursorPosition(end);
        }
    }
}

// Simulate scroll wheel event using macOS CoreGraphics API
pub fn scroll_mouse(delta_y: i32) {
    unsafe {
        let scroll_ev = CGEventCreateScrollWheelEvent2(
            std::ptr::null(),
            0, // kCGScrollEventUnitPixel = 0, or kCGScrollEventUnitLine = 1
            1,
            delta_y,
            0,
            0,
        );
        if !scroll_ev.is_null() {
            CGEventPost(0 /* kCGHIDEventTap */, scroll_ev);
            CFRelease(scroll_ev);
        }
    }
}
