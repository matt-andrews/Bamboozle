using Fluid;

namespace Bamboozle.Utilities;

public static class Liquid
{
    private static readonly FluidParser _parser = new();
    public static string? Parse(string input, object context, out string? error)
    {
        if (_parser.TryParse(input, out IFluidTemplate template, out error))
        {
            var templateContext = new TemplateContext(context);
            return template.Render(templateContext);
        }
        return null;
    }

    public static string? LiquidParse(this string input, object context, out string? error)
    {
        return Parse(input, context, out error);
    }
}
