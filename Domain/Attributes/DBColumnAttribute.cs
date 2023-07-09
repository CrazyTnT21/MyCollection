﻿using System;

namespace Domain.Attributes;

[AttributeUsage(AttributeTargets.Property)]
public sealed class DBColumnAttribute : Attribute
{
  public readonly string column;
  public readonly string? foreignTable;

  public DBColumnAttribute(string column, string? foreignTable = null)
  {
    this.column = column;
    this.foreignTable = foreignTable;
  }
}
