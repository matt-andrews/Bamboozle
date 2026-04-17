using System.Diagnostics.CodeAnalysis;
using System.Linq.Dynamic.Core;
using System.Linq.Expressions;

namespace Bamboozle.Utilities.JsonConverters
{
	public static class FilterParser
	{
		/// <summary>
		/// Parses a Dynamic LINQ string into a strongly-typed Expression<Func<T, bool>>.
		/// Throws a FilterParseException if the expression is invalid.
		/// </summary>
		public static Expression<Func<T, bool>> Parse<T>(string filter)
		{
			try
			{
				// Dynamic LINQ compiles the string into a real expression tree
				return DynamicExpressionParser.ParseLambda<T, bool>(
					ParsingConfig.Default,
					createParameterCtor: false,
					filter
				);
			}
			catch (Exception ex)
			{
				throw new FilterParseException($"Invalid filter expression: {ex.Message}", ex);
			}
		}

		public static bool TryParse<T>(
			string filter,
			[NotNullWhen(true)]
			out Expression<Func<T, bool>>? expression,
			[NotNullWhen(false)]
			out string? error)
		{
			try
			{
				expression = Parse<T>(filter);
				error = null;
				return true;
			}
			catch (FilterParseException ex)
			{
				expression = null;
				error = ex.Message;
				return false;
			}
		}
	}


	public class FilterParseException(string message, Exception? inner = null)
		: Exception(message, inner);
}
