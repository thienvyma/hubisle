export interface FriendLabelLayout {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** Place a friend name beside its dot, toward the minimap centre. */
export function friendLabelLayout(
  markerX: number,
  markerY: number,
  center: number,
  radius: number,
  textWidth: number,
): FriendLabelLayout {
  const width = Math.ceil(Math.max(0, textWidth)) + 10;
  const height = 17;
  const left = markerX <= center ? markerX + 9 : markerX - 9 - width;
  const top = Math.max(
    center - radius + 3,
    Math.min(markerY - height / 2, center + radius - height - 3),
  );
  return { left, top, width, height };
}
