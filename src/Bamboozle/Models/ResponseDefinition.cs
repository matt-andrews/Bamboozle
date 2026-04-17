using Bamboozle.Utilities;

namespace Bamboozle.Models
{
    public record ResponseDefinition
    {
        public string Status { get; init; } = "200";
        public Dictionary<string, string> Headers { get; init; } = [];
        public string? Content { get; init; }

        public ResponseDefinition WithTemplate(ContextModel context)
        {
            return this with
            {
                Status = Status.LiquidParse(context, out _) ?? "200",
                Headers = Headers.ToDictionary(
                    k => k.Key.LiquidParse(context, out _) ?? "",
                    v => v.Value.LiquidParse(context, out _) ?? "") ?? [],
                Content = Content?.LiquidParse(context, out _)
            };
        }
    }
}
