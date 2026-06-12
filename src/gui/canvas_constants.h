// Shared layout / routing constants for the canvas. Pure values; no Qt
// types referenced here so this header can be included from any GUI .cpp
// without dragging in Qt.

#pragma once

namespace hdlc {

inline constexpr char kModuleMimeType[] = "application/x-hdl-compose-module";

// --- Instance geometry ---
inline constexpr int kMinInstanceWidth = 200;
inline constexpr int kInstanceHeaderHeight = 50;
inline constexpr int kPinSlotHeight = 24;
inline constexpr int kPinShapeSize = 12;
inline constexpr int kMinInstanceBodyHeight = 30;
inline constexpr int kPinLabelHPadding = 8;
inline constexpr int kInstanceCenterPadding = 24;

// --- Column / wire grid ---
inline constexpr int kColumnPitch = 480;
inline constexpr int kColumnGutterHalf = 60;
// Cap instance width below the column pitch so a module body can never
// invade the wire gutters on either side of its column.
inline constexpr int kMaxInstanceWidth = kColumnPitch - 2 * kColumnGutterHalf;
inline constexpr int kWireLaneStep = 12;
inline constexpr int kMinModuleVerticalGap = 60;
inline constexpr int kWireStubMin = 40;
inline constexpr int kJunctionDotRadius = 4;

// --- Canvas / view ---
inline constexpr int kClickThresholdPx = 5;
inline constexpr double kZoomMin = 0.2;
inline constexpr double kZoomMax = 5.0;
inline constexpr double kZoomStep = 1.15;

// --- Top-level ports ---
inline constexpr int kTopPortSpacing = 44;

} // namespace hdlc
