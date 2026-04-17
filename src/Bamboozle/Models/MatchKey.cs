namespace Bamboozle.Models;

public record MatchKey(string Verb, string Pattern)
{
    public override string ToString()
    {
        return $"{Verb}|{Pattern}";
    }
}
