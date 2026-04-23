using System.Linq.Expressions;

namespace Bamboozle.Core;

public class BamboozleAssertBuilder
{
    private readonly List<string> _expressions = [];

    public BamboozleAssertBuilder With(Expression<Func<AssertContext, bool>> expression)
    {
        _expressions.Add(ParseExpression(expression.Body));
        return this;
    }

    public override string ToString()
    {
        if (_expressions.Count == 0) return "";
        if (_expressions.Count == 1) return _expressions[0];
        return string.Join(" && ", _expressions.Select(e => $"({e})"));
    }

    private string ParseExpression(Expression expr)
    {
        if (expr is BinaryExpression binary)
        {
            string left = ParseExpression(binary.Left);
            string right = ParseExpression(binary.Right);
            string op = binary.NodeType switch
            {
                ExpressionType.Equal => "==",
                ExpressionType.NotEqual => "!=",
                ExpressionType.GreaterThan => ">",
                ExpressionType.GreaterThanOrEqual => ">=",
                ExpressionType.LessThan => "<",
                ExpressionType.LessThanOrEqual => "<=",
                ExpressionType.AndAlso => "&&",
                ExpressionType.OrElse => "||",
                _ => throw new NotSupportedException($"Operator {binary.NodeType} is not supported in assertions.")
            };
            return $"{left} {op} {right}";
        }

        if (expr is MemberExpression member)
        {
            if (member.Expression is ParameterExpression)
            {
                // Accessing a property on AssertContext
                return member.Member.Name switch
                {
                    nameof(AssertContext.Verb) => "verb",
                    nameof(AssertContext.Pattern) => "pattern",
                    nameof(AssertContext.State) => "state",
                    nameof(AssertContext.BodyValue) => "body",
                    nameof(AssertContext.BodyRaw) => "body_raw",
                    _ => throw new NotSupportedException($"Property {member.Member.Name} is not supported.")
                };
            }
            
            // For closure variables or static members, evaluate the value
            return FormatValue(Evaluate(expr));
        }

        if (expr is MethodCallExpression methodCall)
        {
            if (methodCall.Object is ParameterExpression)
            {
                // Calling a method on AssertContext
                string arg = ParseExpression(methodCall.Arguments[0]);
                return methodCall.Method.Name switch
                {
                    nameof(AssertContext.Query) => $"query({arg})",
                    nameof(AssertContext.Header) => $"header({arg})",
                    nameof(AssertContext.Route) => $"route({arg})",
                    nameof(AssertContext.Body) => $"body({arg})",
                    _ => throw new NotSupportedException($"Method {methodCall.Method.Name} is not supported.")
                };
            }
            
            // Checking if it is an extension method or instance method on string
            if (methodCall.Method.DeclaringType == typeof(string))
            {
                string caller = ParseExpression(methodCall.Object!);
                bool isSupportedStringMethod =
                   methodCall.Method.Name is nameof(string.Contains) or nameof(string.StartsWith) or nameof(string.EndsWith);
                if (isSupportedStringMethod)
                {
                    if (methodCall.Arguments.Count != 1 || methodCall.Method.GetParameters().Length != 1)
                    {
                        throw new NotSupportedException(
                            $"String method overload {methodCall.Method.Name} with {methodCall.Method.GetParameters().Length} parameter(s) is not supported. Only the single-string-argument overload is supported.");
                    }
                    string arg = ParseExpression(methodCall.Arguments[0]);
                    return methodCall.Method.Name switch
                    {
                        nameof(string.Contains) => $"contains({caller}, {arg})",
                        nameof(string.StartsWith) => $"starts_with({caller}, {arg})",
                        nameof(string.EndsWith) => $"ends_with({caller}, {arg})",
                        _ => throw new NotSupportedException($"String method {methodCall.Method.Name} is not supported.")
                    };
                }
                throw new NotSupportedException($"String method {methodCall.Method.Name} is not supported.");
            }
            
            // Unhandled method call, we try to evaluate it if it doesn't depend on the parameter
            return FormatValue(Evaluate(expr));
        }

        if (expr is ConstantExpression constant)
        {
            return FormatValue(constant.Value);
        }

        if (expr is UnaryExpression unary)
        {
            if (unary.NodeType == ExpressionType.Not)
            {
                return $"!({ParseExpression(unary.Operand)})";
            }
            if (unary.NodeType == ExpressionType.Convert)
            {
                return ParseExpression(unary.Operand);
            }
        }

        throw new NotSupportedException($"Expression type {expr.NodeType} is not supported.");
    }

    private static object? Evaluate(Expression expr)
    {
        try
        {
            var lambda = Expression.Lambda(expr);
            var compiled = lambda.Compile();
            return compiled.DynamicInvoke();
        }
        catch (Exception ex)
        {
            throw new InvalidOperationException($"Could not evaluate expression: {expr}", ex);
        }
    }

    private static string FormatValue(object? value)
    {
        if (value is string s)
        {
            return $"\"{s.Replace("\"", "\\\"")}\"";
        }
        if (value is bool b)
        {
            return b ? "true" : "false";
        }
        if (value is null)
        {
            return "\"\"";
        }
        if (value is float f)
        {
            return f.ToString(null, System.Globalization.CultureInfo.InvariantCulture);
        }
        if (value is double d)
        {
            return d.ToString(null, System.Globalization.CultureInfo.InvariantCulture);
        }
        if (value is decimal m)
        {
            return m.ToString(null, System.Globalization.CultureInfo.InvariantCulture);
        }
        if (value is IFormattable formattable)
        {
            return formattable.ToString(null, System.Globalization.CultureInfo.InvariantCulture) ?? "\"\"";
        }
        return value.ToString() ?? "\"\"";
    }
}
