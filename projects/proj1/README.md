# Raport 
W makefilu znajduje się bench odpowiedzialne za właczenie benchmarków oraz plot odpowiedzialne za stworzenie wykresów do wyników. Potrzebne do tego będzie wirtualne środowisko pythona z bibliotekami pandas, mathplotlib i seaborn

## Latency
![](plots/latency_file_4kb_time_vs_block_size.png)

![](plots/latency_file_4kb_time_vs_threads.png)

![](plots/latency_file_4mb_time_vs_block_size.png)

![](plots/latency_file_4mb_time_vs_threads.png)

W przypadku obu testowanych plików widać, że wraz ze wzrostem rozmiaru bloku latencja maleje, aż dla względnie dużych rozmiarów wykres się wypłaszcza. Wynika to z tego, że przy małych rozmiarach bloków większym narzutem jest wywołanie syscalla read i dla małej ilości tych wywołań przestają one być bottleneckiem programu.

Wzrost liczby wątków poprawia wyniki jedynie w przypadku, gdy block size jest względnie mały. Dzieje się tak dlatego, że przy dużych blokach mniejsza liczba wątków jest wystarczająca, a dodawanie większej liczby wątków może dodatkowo spowodować narzut przez przełączanie kontekstu między wątkami.

Posiadanie pliku w pamięci systemu operacyjnego skróci czas latencji. Jeśli jednak rozmiar bloku będzie względnie mały to nie zmieni to wiele, bo i tak problemem będzie samo wielokrotne wykonywanie syscalla read.

## Throughput
![](plots/throughput_file_4kb_time_vs_block_size.png)

![](plots/throughput_file_4kb_time_vs_block_size.png)

![](plots/through_old_file_4kb_time_vs_block_size.png)

![](plots/through_old_file_4mb_time_vs_block_size.png)

Zwiększanie rozmiaru bloków skraca czas w starej wersji programu, a w nowej nic nie zmienia, ponieważ w starej wersji programu ogranicza to liczbę wywołań syscalla read, a w nowej i tak jest tylko syscall mmap i dalej program czyta z bufora, więc nie ma to dla niego znaczenia. Zmieniłoby się to jednak, gdyby plik nie mieścił się do RAMu, wtedy program nie może załadować całej zawartości naraz i wystąpią page faulty.

Mmap zmniejszy obciążenie procesora ponad read, gdy plik będzie odczytywany w losowych fragmentach. Dla reada to dodatkowe utrudnienie, a dla mmapa nic nie zmienia.

Posiadanie pliku w pamięci systemu operacyjnego skróci czas latencji. Program nie będzie musiał fizycznie odczytywać pamięci masowej, bo mmap będzie mógł brać całą potrzebną zawartość pliku z RAMu.

## Pipeline
![](plots/pipeline_file_4kb_time_vs_block_size.png)

![](plots/pipeline_file_4kb_time_vs_k.png)

![](plots/pipeline_file_4mb_time_vs_block_size.png)

![](plots/pipeline_file_4mb_time_vs_k.png)

Rozmiar bloku znacząco zmniejsza czas dla dużych k, ponieważ zmniejsza to czas potrzebny na synchronizację przez channel. Przy małym k nie ma to dużego znaczenia, bo i tak bottleneckiem jest producent, który ma tylko jakąś prędkość odczytu z dysku.

Tak jak w treści zadania, do pewnego momentu zwiększanie k nic nie zmienia, bo przy małym k liczenie nie zajmuje dużo i nie jest bottleneckiem programu.

Dodanie większej ilości wątków po stronie czytania sekwencyjnego może nawet spowolnić program, bo wtedy są 2 wątki chętne do czytania i trzeba tym zarządzać, a i tak tylko jeden może czytać naraz.

Dodanie wątków do liczenia funkcji hashującej może przyspieszyć działanie programu w przypadkach, gdy bottleneckiem jest k, wtedy będą one znacznie szybciej liczyć te funkcje skrótu, a sam xor jest bardzo szybki.