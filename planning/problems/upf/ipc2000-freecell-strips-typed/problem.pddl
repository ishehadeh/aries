(define (problem freecell_2_1-problem)
 (:domain freecell_2_1-domain)
 (:objects
   sa s2 s0 ha h2 h0 da d2 d0 ca c2 c0 - card
   n8 n7 n6 n5 n4 n3 n13 n12 n11 n9 n10 n1 n2 n0 - num
   s h d c - suit
 )
 (:init (value c0 n0) (value c2 n2) (value ca n1) (value d0 n0) (value d2 n2) (value da n1) (value h0 n0) (value h2 n2) (value ha n1) (value s0 n0) (value s2 n2) (value sa n1) (successor n1 n0) (successor n10 n9) (successor n11 n10) (successor n12 n11) (successor n13 n12) (successor n2 n1) (successor n3 n2) (successor n4 n3) (successor n5 n4) (successor n6 n5) (successor n7 n6) (successor n8 n7) (successor n9 n8) (suit_0 c0 c) (suit_0 c2 c) (suit_0 ca c) (suit_0 d0 d) (suit_0 d2 d) (suit_0 da d) (suit_0 h0 h) (suit_0 h2 h) (suit_0 ha h) (suit_0 s0 s) (suit_0 s2 s) (suit_0 sa s) (canstack ca d2) (canstack ca h2) (canstack da c2) (canstack da s2) (canstack ha c2) (canstack ha s2) (canstack sa d2) (canstack sa h2) (home c0) (home d0) (home h0) (home s0) (cellspace n4) (colspace n2) (on c2 ca) (on ca sa) (clear c2) (clear d2) (clear da) (clear h2) (clear ha) (clear s2) (bottomcol d2) (bottomcol da) (bottomcol h2) (bottomcol ha) (bottomcol s2) (bottomcol sa))
 (:goal (and (home c2) (home d2) (home h2) (home s2)))
)
