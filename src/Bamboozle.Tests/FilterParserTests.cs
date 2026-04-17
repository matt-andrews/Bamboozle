using Bamboozle.Utilities.JsonConverters;
using FluentAssertions;
using System;
using System.Linq;
using System.Linq.Expressions;
using Xunit;

namespace Bamboozle.Tests
{
	public class FilterParserTests
	{
		// -------------------------------------------------------------------------
		// Test models
		// -------------------------------------------------------------------------

		private record Product(string Name, decimal Price, int Stock, bool IsActive, DateTime CreatedAt);

		private static readonly Product[] Products =
		[
			new("Apple",  1.50m,  100, true,  new DateTime(2024, 1, 1)),
		new("Banana", 0.75m,  200, true,  new DateTime(2024, 3, 15)),
		new("Cherry", 3.00m,    0, false, new DateTime(2023, 6, 10)),
		new("Date",   5.00m,   50, true,  new DateTime(2022, 12, 31)),
		new("Elderberry", 8.50m, 10, false, new DateTime(2025, 2, 20)),
	];

		// Helper: compile and apply the parsed expression to the test dataset.
		private static Product[] Filter(string filter)
		{
			var expr = FilterParser.Parse<Product>(filter);
			return Products.Where(expr.Compile()).ToArray();
		}

		// =========================================================================
		// Parse<T> — happy-path tests
		// =========================================================================

		public class Parse_ValidExpressions : FilterParserTests
		{
			[Fact]
			public void Returns_expression_for_simple_equality()
			{
				var expr = FilterParser.Parse<Product>(@"Name == ""Apple""");

				expr.Should().NotBeNull();
				expr.Should().BeAssignableTo<Expression<Func<Product, bool>>>();
			}

			[Fact]
			public void Filters_by_string_equality()
			{
				var results = Filter(@"Name == ""Banana""");

				results.Should().ContainSingle()
					   .Which.Name.Should().Be("Banana");
			}

			[Fact]
			public void Filters_by_numeric_equality()
			{
				var results = Filter("Price == 3.00");

				results.Should().ContainSingle()
					   .Which.Name.Should().Be("Cherry");
			}

			[Fact]
			public void Filters_by_greater_than()
			{
				var results = Filter("Price > 4.00");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Date", "Elderberry");
			}

			[Fact]
			public void Filters_by_less_than_or_equal()
			{
				var results = Filter("Price <= 1.50");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Apple", "Banana");
			}

			[Fact]
			public void Filters_by_boolean_property_true()
			{
				var results = Filter("IsActive == true");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Apple", "Banana", "Date");
			}

			[Fact]
			public void Filters_by_boolean_property_false()
			{
				var results = Filter("IsActive == false");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Cherry", "Elderberry");
			}

			[Fact]
			public void Filters_by_integer_property()
			{
				var results = Filter("Stock == 0");

				results.Should().ContainSingle()
					   .Which.Name.Should().Be("Cherry");
			}

			[Fact]
			public void Filters_using_AND_conjunction()
			{
				var results = Filter("IsActive == true AND Price < 2.00");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Apple", "Banana");
			}

			[Fact]
			public void Filters_using_OR_disjunction()
			{
				var results = Filter(@"Name == ""Apple"" OR Name == ""Date""");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Apple", "Date");
			}

			[Fact]
			public void Filters_using_NOT_operator()
			{
				var results = Filter("NOT (IsActive == true)");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Cherry", "Elderberry");
			}

			[Fact]
			public void Handles_combined_AND_OR_with_parentheses()
			{
				// (cheap AND active) OR very expensive
				var results = Filter("(Price < 1.00 AND IsActive == true) OR Price > 7.00");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Banana", "Elderberry");
			}

			[Fact]
			public void Filters_by_DateTime_property()
			{
				var results = Filter("CreatedAt >= DateTime(2024, 1, 1)");

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Apple", "Banana", "Elderberry");
			}

			[Fact]
			public void Handles_string_Contains()
			{
				var results = Filter(@"Name.Contains(""rr"")");  // Cherry, Elderberry

				results.Select(p => p.Name)
					   .Should().BeEquivalentTo("Cherry", "Elderberry");
			}

			[Fact]
			public void Handles_string_StartsWith()
			{
				var results = Filter(@"Name.StartsWith(""A"")");

				results.Should().ContainSingle()
					   .Which.Name.Should().Be("Apple");
			}

			[Fact]
			public void Returns_all_records_for_tautology()
			{
				var results = Filter("Price >= 0");

				results.Should().HaveCount(Products.Length);
			}

			[Fact]
			public void Returns_no_records_for_contradiction()
			{
				var results = Filter("Price > 1000");

				results.Should().BeEmpty();
			}

			[Fact]
			public void Expression_body_is_not_null()
			{
				var expr = FilterParser.Parse<Product>("Stock > 10");

				expr.Body.Should().NotBeNull();
			}

			[Fact]
			public void Expression_has_single_parameter()
			{
				var expr = FilterParser.Parse<Product>("Stock > 10");

				expr.Parameters.Should().ContainSingle();
			}

			[Fact]
			public void Parsed_expression_can_be_compiled_and_invoked()
			{
				var expr = FilterParser.Parse<Product>("Price == 5.00");
				var predicate = expr.Compile();
				var date = new Product("Date", 5.00m, 50, true, new DateTime(2022, 12, 31));

				predicate(date).Should().BeTrue();
			}
		}

		// =========================================================================
		// Parse<T> — exception / error tests
		// =========================================================================

		public class Parse_InvalidExpressions : FilterParserTests
		{
			[Fact]
			public void Throws_FilterParseException_for_unknown_property()
			{
				var act = () => FilterParser.Parse<Product>("NonExistent == 1");

				act.Should().Throw<FilterParseException>();
			}

			[Fact]
			public void Throws_FilterParseException_for_syntax_error()
			{
				var act = () => FilterParser.Parse<Product>("Price === 1.00");   // invalid operator

				act.Should().Throw<FilterParseException>();
			}

			[Fact]
			public void Throws_FilterParseException_for_empty_string()
			{
				var act = () => FilterParser.Parse<Product>(string.Empty);

				act.Should().Throw<FilterParseException>();
			}

			[Fact]
			public void Throws_FilterParseException_for_type_mismatch()
			{
				// Comparing a numeric property to a plain string literal
				var act = () => FilterParser.Parse<Product>(@"Price == ""not-a-number""");

				act.Should().Throw<FilterParseException>();
			}

			[Fact]
			public void Thrown_exception_message_contains_original_detail()
			{
				var act = () => FilterParser.Parse<Product>("BadField > 0");

				act.Should().Throw<FilterParseException>()
				   .WithMessage("*Invalid filter expression*");
			}

			[Fact]
			public void Thrown_exception_has_inner_exception()
			{
				var act = () => FilterParser.Parse<Product>("BadField > 0");

				act.Should().Throw<FilterParseException>()
				   .Which.InnerException.Should().NotBeNull();
			}

			[Fact]
			public void Throws_FilterParseException_for_unbalanced_parentheses()
			{
				var act = () => FilterParser.Parse<Product>("(Price > 1");

				act.Should().Throw<FilterParseException>();
			}
		}

		// =========================================================================
		// TryParse<T> — success path
		// =========================================================================

		public class TryParse_SuccessPath : FilterParserTests
		{
			[Fact]
			public void Returns_true_for_valid_filter()
			{
				var result = FilterParser.TryParse<Product>("Stock > 0", out _, out _);

				result.Should().BeTrue();
			}

			[Fact]
			public void Outputs_non_null_expression_on_success()
			{
				FilterParser.TryParse<Product>("Stock > 0", out var expr, out _);

				expr.Should().NotBeNull();
			}

			[Fact]
			public void Outputs_null_error_on_success()
			{
				FilterParser.TryParse<Product>("Stock > 0", out _, out var error);

				error.Should().BeNull();
			}

			[Fact]
			public void Returned_expression_produces_correct_results()
			{
				FilterParser.TryParse<Product>("IsActive == true", out var expr, out _);

				var results = Products.Where(expr!.Compile()).ToArray();

				results.Should().HaveCount(3);
			}

			[Fact]
			public void Succeeds_for_complex_valid_expression()
			{
				var result = FilterParser.TryParse<Product>(
					"Price > 1.00 AND IsActive == true",
					out var expr,
					out var error);

				result.Should().BeTrue();
				expr.Should().NotBeNull();
				error.Should().BeNull();
			}
		}

		// =========================================================================
		// TryParse<T> — failure path
		// =========================================================================

		public class TryParse_FailurePath : FilterParserTests
		{
			[Fact]
			public void Returns_false_for_invalid_filter()
			{
				var result = FilterParser.TryParse<Product>("!!!!", out _, out _);

				result.Should().BeFalse();
			}

			[Fact]
			public void Outputs_null_expression_on_failure()
			{
				FilterParser.TryParse<Product>("BadField == 1", out var expr, out _);

				expr.Should().BeNull();
			}

			[Fact]
			public void Outputs_non_null_error_message_on_failure()
			{
				FilterParser.TryParse<Product>("BadField == 1", out _, out var error);

				error.Should().NotBeNullOrWhiteSpace();
			}

			[Fact]
			public void Error_message_contains_useful_text_on_failure()
			{
				FilterParser.TryParse<Product>("BadField == 1", out _, out var error);

				error.Should().Contain("Invalid filter expression");
			}

			[Fact]
			public void Returns_false_for_empty_string()
			{
				var result = FilterParser.TryParse<Product>(string.Empty, out _, out _);

				result.Should().BeFalse();
			}

			[Fact]
			public void Does_not_throw_for_any_invalid_input()
			{
				var inputs = new[] { "???", "Price === 1", "Name.Fake()", string.Empty, "   " };

				foreach (var input in inputs)
				{
					var act = () => FilterParser.TryParse<Product>(input, out _, out _);
					act.Should().NotThrow(because: $"TryParse should swallow errors (input: '{input}')");
				}
			}
		}

		// =========================================================================
		// FilterParseException construction
		// =========================================================================

		public class FilterParseException_Construction
		{
			[Fact]
			public void Can_be_constructed_with_message_only()
			{
				var ex = new FilterParseException("oops");

				ex.Message.Should().Be("oops");
				ex.InnerException.Should().BeNull();
			}

			[Fact]
			public void Can_be_constructed_with_inner_exception()
			{
				var inner = new InvalidOperationException("root cause");
				var ex = new FilterParseException("oops", inner);

				ex.InnerException.Should().BeSameAs(inner);
			}

			[Fact]
			public void Is_assignable_to_Exception()
			{
				var ex = new FilterParseException("oops");

				ex.Should().BeAssignableTo<Exception>();
			}
		}
	}
}
