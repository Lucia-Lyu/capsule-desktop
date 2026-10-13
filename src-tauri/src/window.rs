use tauri::{LogicalSize, PhysicalPosition, PhysicalSize, WebviewWindow};

const COLLAPSED_SIZE: (u32, u32) = (110, 32);
const ATTENTION_SIZE: (u32, u32) = (280, 140);

fn centered_position(
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    window_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    PhysicalPosition::new(
        monitor_position.x + (monitor_size.width as i32 - window_size.width as i32) / 2,
        monitor_position.y + (monitor_size.height as i32 - window_size.height as i32) / 2,
    )
}

fn collapsed_return_position(
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    old_position: PhysicalPosition<i32>,
    old_size: PhysicalSize<u32>,
    collapsed_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    let monitor_middle = monitor_position.x + monitor_size.width as i32 / 2;
    let old_middle = old_position.x + old_size.width as i32 / 2;
    let x = if old_middle > monitor_middle {
        old_position.x + old_size.width as i32 - collapsed_size.width as i32
    } else {
        old_position.x
    };
    PhysicalPosition::new(x, old_position.y)
}

pub fn center_attention(window: &WebviewWindow) -> tauri::Result<PhysicalPosition<i32>> {
    let original = window.outer_position()?;
    let original_size = window.outer_size()?;
    let monitor = window.current_monitor()?.or(window.primary_monitor()?);
    let scale = monitor.as_ref().map(|monitor| monitor.scale_factor()).unwrap_or(window.scale_factor()?);
    let size: PhysicalSize<u32> = LogicalSize::new(ATTENTION_SIZE.0, ATTENTION_SIZE.1).to_physical(scale);
    let collapsed_size: PhysicalSize<u32> = LogicalSize::new(COLLAPSED_SIZE.0, COLLAPSED_SIZE.1).to_physical(scale);
    let return_position = monitor.as_ref().map(|monitor| {
        collapsed_return_position(*monitor.position(), *monitor.size(), original, original_size, collapsed_size)
    }).unwrap_or(original);
    window.set_size(size)?;
    if let Some(monitor) = monitor {
        window.set_position(centered_position(*monitor.position(), *monitor.size(), size))?;
    }
    Ok(return_position)
}

pub fn restore_attention(window: &WebviewWindow, original: PhysicalPosition<i32>) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let size: PhysicalSize<u32> = LogicalSize::new(COLLAPSED_SIZE.0, COLLAPSED_SIZE.1).to_physical(scale);
    window.set_size(size)?;
    window.set_position(original)?;
    Ok(())
}

pub fn position_initial(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let size = window.outer_size()?;
        let margin = (18.0 * monitor.scale_factor()).round() as i32;
        let x = monitor.position().x + monitor.size().width as i32 - size.width as i32 - margin;
        let y = monitor.position().y + (monitor.size().height as f64 * 0.22).round() as i32;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

pub fn resize_capsule(window: &WebviewWindow, width: u32, height: u32) -> tauri::Result<()> {
    let old_position = window.outer_position()?;
    let old_size = window.outer_size()?;
    let scale = window.scale_factor()?;
    let new_size: PhysicalSize<u32> = LogicalSize::new(width, height).to_physical(scale);

    let monitor = window.current_monitor()?.or(window.primary_monitor()?);
    let mut x = old_position.x;
    let mut y = old_position.y;

    if let Some(monitor) = monitor {
        let left = monitor.position().x;
        let top = monitor.position().y;
        let right = left + monitor.size().width as i32;
        let bottom = top + monitor.size().height as i32;
        let was_on_right = old_position.x + old_size.width as i32 / 2 > (left + right) / 2;

        // Keep the edge nearest the screen edge in place while the capsule changes size.
        if was_on_right {
            x = old_position.x + old_size.width as i32 - new_size.width as i32;
        }
        x = x.clamp(left, (right - new_size.width as i32).max(left));
        y = y.clamp(top, (bottom - new_size.height as i32).max(top));
    }

    window.set_size(new_size)?;
    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reminder_is_centered_on_capsules_monitor_with_negative_coordinates() {
        let position = centered_position(
            PhysicalPosition::new(-1920, 100),
            PhysicalSize::new(1920, 1080),
            PhysicalSize::new(280, 140),
        );
        assert_eq!(position, PhysicalPosition::new(-1100, 570));
    }

    #[test]
    fn expanded_capsule_returns_to_the_same_right_edge() {
        let position = collapsed_return_position(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            PhysicalPosition::new(1620, 180),
            PhysicalSize::new(280, 360),
            PhysicalSize::new(110, 32),
        );
        assert_eq!(position, PhysicalPosition::new(1790, 180));
    }
}
