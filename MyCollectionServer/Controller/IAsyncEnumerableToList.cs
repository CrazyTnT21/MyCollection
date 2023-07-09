﻿using System.Collections.Generic;
using System.Threading.Tasks;

namespace MyCollectionServer.Controller;

public static class IAsyncEnumerableToList
{
  public static async Task<List<T>> ToList<T>(this IAsyncEnumerable<T> enumerable)
  {
    var result = new List<T>();
    var enumerator = enumerable.GetAsyncEnumerator();

    while (await enumerator.MoveNextAsync())
    {
      result.Add(enumerator.Current);
    }

    return result;
  }
}
