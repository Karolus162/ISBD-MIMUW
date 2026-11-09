# Przykład i testy

#### 1. Przykładowy plik można wygenerować poprzez `cargo run --release --bin gen_example <filepath>`

#### 2. Program liczący metryki z pliku można wywołać poprzez `cargo run --release --bin calculate_metrics <filepath>`

Wygenerowany przez mnie plik znajduje się w `files/plik`

#### 3. Testy wykonywane są poprzez `make test`

# Dokumentacja Formatu Pliku

## 1. Nagłówek pliku (Schemat kolumn)

Na samym początku pliku znajduje się sekwencja bajtów definiująca układ kolumn w każdym batchu. 

* **Oznaczenia bajtów:**
  * `0` – Kolumna typu `INT`
  * `1` – Kolumna typu `BOOL`
  * `2` – Kolumna typu `VARCHAR`
* **Zakończenie nagłówka:** Ciąg bajtów schematu kończy się bajtem `3`.

 **Przykład:**  
 Nagłówek `0, 2, 1, 2, 3` oznacza, że w każdym batchu dane występują w następującej kolejności:
 1. `INT`
 2. `VARCHAR`
 3. `BOOL`
 4. `VARCHAR`

Po nagłówku, aż do końca pliku, następuje sekwencja kolejnych batchy danych.

---

## 2. Struktura batcha

Każdy batch składa się z dwóch części:

1. **Liczba wierszy (`row_count`):** Pierwsze **2 bajty** określają liczbę wierszy w danym batchu.  
   (Domyślny `BATCH_SIZE` ustawiany na 1024, w testach i przykładzie może być mniejszy).
2. **Dane kolumn:** Następnie zapisane są dane poszczególnych kolumn, według schematu zdefiniowanego w nagłówku pliku.

---

## 3. Opis kolumn

### 3.1. Kolumna INT

* **Rozmiar danych:** Pierwsze **2 bajty** określają rozmiar w bajtach dla danych tej kolumny w danym batchu.
* **Kodowanie liczb:** Wpisy to `row_count` liczb całkowitych kodowanych następująco:
  * Pierwsza liczba w batchu jest zapisana normalnie.
  * Każda kolejna liczba jest kodowana jako delta (różnica względem poprzedniej wartości w kolumnie).
  * **Struktura bajtu:**
    * Najstarszy bit informuje, czy kodowanie danej liczby kontynuuje się w kolejnym bajcie (`1` = kontynuacja, `0` = koniec).
    * Drugi od góry bit na pierwszym bajcie informuje, czy liczba jest ujemna (`1` = ujemna, `0` = dodatnia).
    * Pozostałe 6 bitów pierwszego bajtu (lub 7 bitów w kolejnych bajtach) przechowuje właściwe bity liczby.
* Delty pomiędzy kolejnymi liczbami wykraczają poza `i64`, mieszcząc się w przedziale od $-2^{64} + 1$ do $2^{64} - 1$. Pojedyncza wartość zajmuje maksymalnie $1 + \lceil \frac{64-6}{7} \rceil = 10$ bajtów. Dla `BATCH_SIZE = 1024` maksymalny rozmiar danych kolumny wynosi $1024 \times 10$ bajtów, więc 2 bajty na nagłówek rozmiaru są wystarczające.

### 3.2. Kolumna BOOL

* **Kodowanie:** Wartości logiczne są spakowane bitowo. Zajmują dokładnie $\lceil \mathit{row\_count} / 8 \rceil$ bajtów.
* Dla n-tego boola (indeksowanego od 0):
  * Znajduje się on na bajcie o indeksie $\lfloor n / 8 \rfloor$.
  * Pozycja bitu wewnątrz tego bajtu to $n \pmod 8$.

### 3.3. Kolumna VARCHAR

* **Rozmiar danych:** Pierwsze **4 bajty** określają rozmiar w bajtach dla danych tej kolumny w danym batchu.
* **Tablica długości:** Następnie zapisana jest tablica długości kolejnych stringów (w bajtach), zakodowana w taki sam sposób jak liczby w kolumnie `INT`.
* **Kompresja danych:** Pozostała część bufora to ciąg znaków skompresowany algorytmem ZSTD (poziom 3).
* Zakładam, że pojedynczy string nie przekracza rozsądnych rozmiarów (np. $2^{20}$ bajtów), dzięki czemu 4 bajty na nagłówek rozmiaru są w zupełności wystarczające.
