namespace IslemapThienvyma.Telemetry;

public readonly record struct WorldLocation
{
    public double X { get; init; }
    public double Y { get; init; }
    public double Z { get; init; }
}

public static class MapHeading
{
    public static double FromUnrealYaw(double yawDegrees) => Normalize(yawDegrees + 90d);

    public static double Normalize(double degrees)
    {
        var normalized = degrees % 360d;
        return normalized < 0d ? normalized + 360d : normalized;
    }
}
