namespace Bamboozle.Models;

public record MatchKey
{
    public string Verb { get; set; }
    public string Pattern { get; set; }
    public MatchKey(string verb, string pattern)
    {
        Verb = verb;
        Pattern = pattern;
    }
	public MatchKey() { }

	public override string ToString()
    {
        return $"{Verb}|{Pattern}";
    }
}
