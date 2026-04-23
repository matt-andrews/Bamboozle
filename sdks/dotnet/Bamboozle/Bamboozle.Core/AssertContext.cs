namespace Bamboozle.Core;

/// <summary>
/// Represents the context used in an assert expression.
/// These properties and methods map directly to the variables and functions 
/// available in the Bamboozle Rust evalexpr backend.
/// </summary>
public class AssertContext
{
    // Variables
    public string Verb { get; } = "";
    public string Pattern { get; } = "";
    public string State { get; } = "";
    
    /// <summary>
    /// Maps to the "body" variable, which holds the stringified JSON or raw body.
    /// </summary>
    public string BodyValue { get; } = ""; 
    
    /// <summary>
    /// Maps to the "body_raw" variable, the raw unparsed string body.
    /// </summary>
    public string BodyRaw { get; } = "";

    // Functions
    public string Query(string key) => "";
    public string Header(string key) => "";
    public string Route(string key) => "";
    public string Body(string key) => "";
}
